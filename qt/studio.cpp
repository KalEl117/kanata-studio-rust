#include "studio.h"
#include "preview.h"
#include "rust_core.h"

#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QProcess>
#include <QProcessEnvironment>
#include <QRegularExpression>
#include <QStandardPaths>
#include <QDateTime>
#include <QMap>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QtConcurrent/QtConcurrentRun>
#include <algorithm>
#include <signal.h>
#include <functional>

static QByteArray digest(const QByteArray &data) { return QCryptographicHash::hash(data, QCryptographicHash::Sha256); }

Studio::Studio(QObject *parent) : QObject(parent), m_rustEditor(ks_editor_new()) {
    connect(&m_environmentWatcher, &QFutureWatcher<QJsonObject>::finished, this, [this] {
        const QJsonObject result = m_environmentWatcher.result();
        const QString choice = result.value("layout").toObject().value("choice").toString();
        if (choice != m_layoutInfo.value("choice").toString()) { detectKeyboardLayout(); return; }
        applyEnvironment(result);
    });
    applyEnvironment(rustRequest({{"op", "environment"}}));
    m_previewTimer.setInterval(250);
    connect(&m_previewTimer, &QTimer::timeout, this, &Studio::updatePreview);
    const QString dir = QDir::homePath() + "/.config/kanata/";
    const QString preferred = dir + "kanata.kbd";
    const QString alternate = dir + "config.kbd";
    const QString service = QFile::exists(preferred) || QFile::exists(alternate) ? QString() : defaultConfigPath();
    if (QFile::exists(preferred) || QFile::exists(alternate) || QFile::exists(service))
        open(QFile::exists(preferred) ? preferred : QFile::exists(alternate) ? alternate : service);
    else
        setStatus("Keine Kanata-Konfiguration gefunden");
    m_environmentTimer.setInterval(2500);
    connect(&m_environmentTimer, &QTimer::timeout, this, [this] { detectKeyboardLayout(); });
    m_environmentTimer.start();
    connect(&m_jobWatcher, &QFutureWatcher<JobResult>::finished, this, [this] {
        finishJob(m_job, m_jobWatcher.result());
        emit busyChanged();
    });
}

void Studio::applyEnvironment(const QJsonObject &result) {
    if (!result.value("ok").toBool()) return;
    const QVariantMap theme = result.value("theme").toObject().toVariantMap();
    const QVariantMap layout = result.value("layout").toObject().toVariantMap();
    if (theme == m_theme && layout == m_layoutInfo) return;
    const bool newTheme = theme != m_theme, newLayout = layout != m_layoutInfo;
    m_theme = theme; m_layoutInfo = layout; m_keyNames.clear();
    if (!m_source.isEmpty()) refresh();
    if (newTheme) emit themeChanged();
    if (newLayout) emit layoutChanged();
    emit changed();
}

void Studio::setLayoutChoice(const QString &choice) {
    const QJsonObject result = rustRequest({{"op", "layoutChoice"}, {"choice", choice}});
    if (!result.value("ok").toBool()) { setStatus(result.value("error").toString(), true); return; }
    m_layoutInfo.insert("choice", choice); m_keyNames.clear();
    if (!m_source.isEmpty()) refresh();
    emit layoutChanged(); emit changed();
    detectKeyboardLayout();
}

void Studio::detectKeyboardLayout() {
    if (m_environmentWatcher.isRunning()) return;
    const QByteArray request = QJsonDocument(QJsonObject{{"op", "environment"}, {"config", m_original}}).toJson(QJsonDocument::Compact);
    m_environmentWatcher.setFuture(QtConcurrent::run([request] {
        void *editor = ks_editor_new();
        char *response = ks_editor_request(editor, request.constData());
        const QJsonObject result = response ? QJsonDocument::fromJson(QByteArray(response)).object() : QJsonObject{};
        ks_string_free(response); ks_editor_free(editor);
        return result;
    }));
}

QString Studio::effectiveLayout() const {
    const QString choice = m_layoutInfo.value("choice").toString();
    return QStringList{"de", "us", "gb"}.contains(choice) ? choice : m_layoutInfo.value("id", "de").toString();
}
void Studio::setStatus(const QString &message, bool error) {
    if (m_status == message && m_statusError == error) return;
    m_status = message; m_statusError = error; emit statusChanged(); emit changed();
}
void Studio::report(const QString &message) { setStatus(message, true); }

Studio::~Studio() { m_environmentWatcher.waitForFinished(); m_jobWatcher.waitForFinished(); ks_editor_free(m_rustEditor); }

QJsonObject Studio::rustRequest(QJsonObject request) const {
    const QByteArray input = QJsonDocument(request).toJson(QJsonDocument::Compact);
    char *response = ks_editor_request(m_rustEditor, input.constData());
    if (!response) return {{"ok", false}, {"error", "Rust-Kern nicht verfügbar"}};
    const QJsonObject result = QJsonDocument::fromJson(QByteArray(response)).object();
    ks_string_free(response);
    return result;
}

void Studio::clearRustHistory() {
    rustRequest({{"op", "clear"}, {"source", m_source}});
    m_canUndo = false; m_canRedo = false; emit historyChanged();
}

bool Studio::rustEdit(const QString &operation, QJsonObject arguments) {
    arguments.insert("op", operation);
    arguments.insert("source", m_source);
    if (QStringList{"setAction", "setTapHold", "setMacro"}.contains(operation) && m_selectedPhysicalKey.isEmpty()) {
        setStatus("Bitte eine Taste auswählen", true); return false;
    }
    const QString previous = m_source;
    const QJsonObject result = rustRequest(arguments);
    if (!result.value("ok").toBool()) { setStatus(result.value("error").toString(), true); return false; }
    m_source = result.value("source").toString();
    m_canUndo = result.value("canUndo").toBool();
    m_canRedo = result.value("canRedo").toBool(); emit historyChanged();
    applyView(result.value("view").toObject());
    setStatus(m_source == previous ? "Belegung unverändert" : "Entwurf geändert · vor dem Speichern prüfen");
    return true;
}

void Studio::applyView(const QJsonObject &view) {
    const QVariantList keys = view.value("keys").toArray().toVariantList();
    const QVariantMap keyNames = view.value("keyNames").toObject().toVariantMap();
    const bool namesChanged = keyNames != m_keyNamesMap;
    const QVariantList layers = view.value("layers").toArray().toVariantList();
    const QVariantList actions = view.value("actions").toArray().toVariantList();
    const QVariantList labels = view.value("labels").toArray().toVariantList();
    const QVariantMap details = view.value("details").toObject().toVariantMap();
    const QVariantList forms = view.value("configForms").toArray().toVariantList();
    const int layer = view.value("selectedLayer").toInt(), key = view.value("selectedKey").toInt(-1);
    const QString physical = view.value("selectedPhysicalKey").toString();
    const QVariantMap selectedState = view.value("selectedKeyState").toObject().toVariantMap();
    const QVariantList states = view.value("keyStates").toArray().toVariantList();
    const bool document = m_viewSource != m_source;
    const bool newKeys = m_keys != keys, newLayers = m_layers != layers, newForms = m_configForms != forms;
    const bool selection = m_selectedLayer != layer || m_selectedKey != key || m_selectedPhysicalKey != physical;
    const bool actionChange = m_actions != actions || m_actionLabels != labels || m_actionDetails != details || m_selectedKeyState != selectedState || m_keyStates != states || selection;
    m_keys = keys; m_layers = layers; m_actions = actions; m_actionLabels = labels;
    m_actionDetails = details; m_configForms = forms; m_selectedLayer = layer; m_selectedKey = key; m_viewSource = m_source;
    m_keyNamesMap = keyNames;
    m_selectedPhysicalKey = physical; m_selectedKeyState = selectedState; m_keyStates = states;
    if (document) emit documentChanged();
    if (newKeys) emit keysChanged();
    if (namesChanged) emit keyNamesChanged();
    if (newLayers) emit layersChanged();
    if (newForms) emit configFormsChanged();
    if (selection) emit selectionChanged();
    if (actionChange) emit actionsChanged();
}

bool Studio::refresh(QString *error) {
    const QJsonObject result = rustRequest({{"op", "view"}, {"source", m_source}, {"comparisonSource", m_original}, {"layout", effectiveLayout()}});
    if (!result.value("ok").toBool()) { if (error) *error = result.value("error").toString(); return false; }
    applyView(result.value("view").toObject());
    return true;
}

QVariantList Studio::configForms() const { return m_configForms; }
bool Studio::replaceConfigForm(int formIndex, const QString &expression) {
    return rustEdit("replaceConfigForm", {{"formIndex", formIndex}, {"expression", expression}});
}

bool Studio::addConfigForm(const QString &expression) {
    return rustEdit("addConfigForm", {{"expression", expression}});
}

QString Studio::action() const { return m_selectedKey < 0 && !m_selectedPhysicalKey.isEmpty() ? QStringLiteral("_") : actionFor(m_selectedKey); }

QString Studio::actionFor(int key) const {
    return key >= 0 && key < m_actions.size() ? m_actions[key].toString() : QString{};
}

QString Studio::displayKeyName(const QString &key) const {
    if (m_keyNamesMap.contains(key)) return m_keyNamesMap.value(key).toString();
    if (!m_keyNames.contains(key)) {
        const QJsonObject result = rustRequest({{"op", "keyLabel"}, {"key", key}, {"layout", effectiveLayout()}});
        m_keyNames.insert(key, result.value("label").toString());
    }
    return m_keyNames.value(key);
}

QString Studio::actionLabelFor(int key) const {
    return key >= 0 && key < m_actionLabels.size() ? m_actionLabels[key].toString() : QString{};
}
bool Studio::open(const QString &filename) {
    if (busy()) { setStatus("Bitte warten, bis der laufende Vorgang abgeschlossen ist", true); return false; }
    if (m_previewActive) { setStatus("Bitte zuerst den laufenden Test beenden", true); return false; }
    QFile file(filename);
    if (!file.open(QIODevice::ReadOnly)) { setStatus("Datei kann nicht geöffnet werden: " + file.errorString(), true); return false; }
    const QByteArray data = file.readAll();
    // QString::fromUtf8 drops a BOM and replaces invalid bytes; refuse anything a save could not reproduce.
    const QByteArray bomBytes("\xEF\xBB\xBF");
    const bool bom = data.startsWith(bomBytes);
    const QString source = QString::fromUtf8(data);
    if ((bom ? bomBytes : QByteArray()) + source.toUtf8() != data) {
        setStatus("Datei ist kein gültiges UTF-8 und wird zum Schutz nicht geöffnet", true); return false;
    }
    const QJsonObject result = rustRequest({{"op", "loadView"}, {"source", source}, {"layout", effectiveLayout()}});
    if (!result.value("ok").toBool()) { setStatus("Konfiguration nicht lesbar: " + result.value("error").toString(), true); return false; }
    m_source = source; applyView(result.value("view").toObject());
    m_path = QFileInfo(filename).absoluteFilePath(); m_original = m_source; m_diskHash = digest(data); m_bom = bom;
    clearRustHistory(); emit documentChanged();
    setStatus("Konfiguration geladen");
    return true;
}

void Studio::selectLayer(int index) {
    if (index < 0 || index == m_selectedLayer) return;
    const QJsonObject result = rustRequest({{"op", "select"}, {"layer", index}});
    if (result.value("ok").toBool()) { applyView(result.value("view").toObject()); emit changed(); }
}
void Studio::selectKey(int index) {
    if (index < 0 || index == m_selectedKey) return;
    const QJsonObject result = rustRequest({{"op", "select"}, {"key", index}});
    if (result.value("ok").toBool()) { applyView(result.value("view").toObject()); emit changed(); }
}
bool Studio::selectPhysicalKey(const QString &name) {
    const QJsonObject result = rustRequest({{"op", "selectPhysicalKey"}, {"name", name}});
    if (!result.value("ok").toBool()) { setStatus(result.value("error").toString(), true); return false; }
    applyView(result.value("view").toObject());
    return true;
}
QVariantMap Studio::keyStateFor(int index) const {
    return index >= 0 && index < m_keyStates.size() ? m_keyStates[index].toMap() : QVariantMap{{"kind", "normal"}, {"label", "Normale Funktion"}};
}
bool Studio::setAction(const QString &value) {
    return rustEdit("setAction", {{"expression", value}});
}

bool Studio::undo() { return rustEdit("undo"); }
bool Studio::redo() { return rustEdit("redo"); }

QVariantMap Studio::actionDetails() const { return m_actionDetails; }

bool Studio::setTapHold(const QString &head, int repeatMs, int holdMs, const QString &tap, const QString &hold) {
    return rustEdit("setTapHold", {{"head", head}, {"repeatMs", repeatMs}, {"holdMs", holdMs}, {"tap", tap}, {"hold", hold}});
}

bool Studio::setMacro(const QString &head, const QVariantList &steps) {
    return rustEdit("setMacro", {{"head", head}, {"steps", QJsonArray::fromVariantList(steps)}});
}

bool Studio::addPhysicalKey(const QString &name, const QString &action) {
    if (!rustEdit("addPhysicalKey", {{"name", name}, {"expression", action}})) return false;
    return true;
}

bool Studio::addLayer(const QString &name) { return addLayer(name, -1); }

bool Studio::addLayer(const QString &name, int copyFrom) {
    QJsonObject arguments{{"name", name}};
    if (copyFrom >= 0) arguments.insert("copyFrom", copyFrom);
    if (!rustEdit("addLayer", arguments)) return false;
    return true;
}

bool Studio::renameLayer(int layer, const QString &name) {
    return rustEdit("renameLayer", {{"layer", layer}, {"name", name}});
}

bool Studio::deleteLayer(int layer) {
    if (!rustEdit("deleteLayer", {{"layer", layer}})) return false;
    return true;
}

QString Studio::kanataBinary() const {
    const QString local = QDir::homePath() + "/.local/bin/kanata";
    return QFileInfo(local).isExecutable() ? local : QStandardPaths::findExecutable("kanata");
}

QString Studio::serviceConfigPath(const QString &execStart) {
    static const QRegularExpression rx("(?:^|\\s)(?:-c|--cfg|--config)(?:=|\\s+)(?:\"([^\"]+)\"|(\\S+))");
    const auto match = rx.match(execStart);
    if (!match.hasMatch()) return {};
    return match.captured(1).isEmpty() ? match.captured(2) : match.captured(1);
}

// One `systemctl --user show kanata.service` property; `ok` is false if systemctl failed.
QString Studio::serviceProperty(const QString &property, bool *ok) {
    if (ok) *ok = false;
    QProcess process;
    QProcessEnvironment native = QProcessEnvironment::systemEnvironment();
    if (native.contains("KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH")) native.insert("LD_LIBRARY_PATH", native.value("KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH"));
    process.setProcessEnvironment(native);
    process.start("systemctl", {"--user", "show", "kanata.service", "--property=" + property, "--value"});
    if (!process.waitForStarted(3000) || !process.waitForFinished(5000)
        || process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0) return {};
    if (ok) *ok = true;
    return QString::fromUtf8(process.readAllStandardOutput()).trimmed();
}

// The file kanata.service runs, if it names one; otherwise ~/.config/kanata/kanata.kbd.
QString Studio::defaultConfigPath() {
    QString path = serviceConfigPath(serviceProperty("ExecStart"));
    path.replace("%h", QDir::homePath());
    return path.isEmpty() ? QDir::homePath() + "/.config/kanata/kanata.kbd" : path;
}

QVariantMap Studio::setupStatus() const {
    const QString path = defaultConfigPath();
    return {{"configPath", path}, {"configExists", QFileInfo::exists(path)},
            {"kanata", kanataBinary()}, {"service", serviceProperty("LoadState") == "loaded"}};
}

bool Studio::createConfig(const QVariantList &rows) {
    if (busy() || m_previewActive) { setStatus("Bitte warten, bis der laufende Vorgang abgeschlossen ist", true); return false; }
    const QString path = defaultConfigPath();
    QJsonArray keyRows;
    for (const QVariant &row : rows) keyRows.append(QJsonArray::fromStringList(row.toStringList()));
    const QJsonObject result = rustRequest({{"op", "createConfig"}, {"path", path}, {"rows", keyRows}, {"binary", kanataBinary()}});
    if (!result.value("ok").toBool()) { setStatus(result.value("error").toString(), true); return false; }
    if (!open(path)) return false;
    setStatus("Neue Konfiguration angelegt: " + path);
    return true;
}

bool Studio::serviceUsesOpenFile(const QString &path, QString &error) {
    bool read = false;
    const QString execStart = serviceProperty("ExecStart", &read);
    if (!read) { error = "kanata.service konnte nicht gelesen werden"; return false; }
    QString servicePath = serviceConfigPath(execStart);
    if (servicePath.isEmpty()) { error = "Konfigurationspfad von kanata.service ist unbekannt"; return false; }
    servicePath.replace("%h", QDir::homePath());
    if (QFileInfo(servicePath).canonicalFilePath() != QFileInfo(path).canonicalFilePath()) {
        error = "Diese Datei wird von kanata.service nicht verwendet"; return false;
    }
    return true;
}

bool Studio::prepareJob(JobKind kind, Job &job) {
    if (busy()) { setStatus("Ein Vorgang läuft bereits", true); return false; }
    if (m_path.isEmpty()) { setStatus("Keine Konfiguration geöffnet", true); return false; }
    if (kind == JobKind::Save) {
        if (m_previewActive) { setStatus("Bitte zuerst den laufenden Test beenden", true); return false; }
        if (!dirty()) { setStatus("Keine Änderungen zum Speichern"); return false; }
    }
    if (kind == JobKind::Preview) {
        if (!m_previewReady) { setStatus("Wiederherstellungsdienst ist nicht bereit", true); return false; }
        if (m_previewActive) { setStatus("Ein Test läuft bereits", true); return false; }
    }
    job = {kind, kanataBinary(), m_path, diskText(m_source), runtimeLauncher(), m_diskHash};
    return true;
}

// Runs on a worker thread: touches no Studio state and uses its own Rust handle.
Studio::JobResult Studio::runJob(const Job &job) {
    if (job.binary.isEmpty()) return {false, "Kanata wurde nicht gefunden"};
    const auto rust = [](const QJsonObject &request) {
        void *editor = ks_editor_new();
        char *response = ks_editor_request(editor, QJsonDocument(request).toJson(QJsonDocument::Compact).constData());
        const QJsonObject result = response ? QJsonDocument::fromJson(QByteArray(response)).object()
                                            : QJsonObject{{"ok", false}, {"error", "Rust-Kern nicht verfügbar"}};
        ks_string_free(response); ks_editor_free(editor);
        return result;
    };
    const QJsonObject checked = rust({{"op", "check"}, {"binary", job.binary}, {"path", job.path}, {"source", job.source}});
    if (!checked.value("ok").toBool()) return {false, "Prüfung fehlgeschlagen: " + checked.value("error").toString().left(400)};
    if (job.kind == JobKind::Check) return {true, "Kanata Prüfung erfolgreich"};
    QString error;
    if (!serviceUsesOpenFile(job.path, error)) return {false, error};
    QFile file(job.path);
    if (!file.open(QIODevice::ReadOnly)) return {false, "Konfiguration kann nicht erneut gelesen werden"};
    const QByteArray current = file.readAll(); file.close();
    if (digest(current) != job.diskHash) return {false, "Datei wurde extern geändert · bitte neu öffnen"};
    if (job.kind == JobKind::Preview) {
        if (!Preview::start(job.launcher, job.path, current, job.source.toUtf8(), error))
            return {false, "Test konnte nicht gestartet werden: " + error};
        return {true, "Entwurf wird 30 Sekunden getestet · gespeicherte Belegung kehrt automatisch zurück"};
    }
    const QJsonObject saved = rust({{"op", "save"}, {"path", job.path}, {"source", job.source},
                                    {"original", QString::fromLatin1(current.toBase64())}});
    if (!saved.value("ok").toBool()) return {false, saved.value("error").toString()};
    return {true, "Gespeichert und Kanata neu geladen"};
}

bool Studio::finishJob(const Job &job, const JobResult &result) {
    setStatus(result.status, !result.ok);
    if (!result.ok) return false;
    if (job.kind == JobKind::Save) {
        // The user may have kept editing while the save ran; only the saved text becomes the baseline.
        const QString saved = m_bom ? job.source.mid(1) : job.source;
        m_original = saved; m_diskHash = digest(job.source.toUtf8());
        if (m_source == saved) clearRustHistory();
        refresh(); emit documentChanged(); emit changed();
    } else if (job.kind == JobKind::Preview) {
        m_previewActive = true;
        m_previewRecoveryStarted = false;
        m_previewRemaining = 30;
        m_previewTimer.start(); emit previewChanged();
    }
    return true;
}

bool Studio::check() {
    Job job;
    return prepareJob(JobKind::Check, job) && finishJob(job, runJob(job));
}

bool Studio::save() {
    if (!dirty() && !m_path.isEmpty() && !m_previewActive) { setStatus("Keine Änderungen zum Speichern"); return true; }
    Job job;
    return prepareJob(JobKind::Save, job) && finishJob(job, runJob(job));
}

bool Studio::startPreview() {
    Job job;
    return prepareJob(JobKind::Preview, job) && finishJob(job, runJob(job));
}

void Studio::requestJob(JobKind kind) {
    if (!prepareJob(kind, m_job)) return;
    setStatus(kind == JobKind::Check ? "Kanata prüft den Entwurf …" : kind == JobKind::Save ? "Speichern …" : "Test wird gestartet …");
    m_jobWatcher.setFuture(QtConcurrent::run(&Studio::runJob, m_job));
    emit busyChanged();
}

void Studio::requestCheck() { requestJob(JobKind::Check); }
void Studio::requestSave() {
    if (!dirty() && !m_path.isEmpty() && !m_previewActive) { setStatus("Keine Änderungen zum Speichern"); return; }
    requestJob(JobKind::Save);
}
void Studio::requestPreview() { requestJob(JobKind::Preview); }

void Studio::discard() {
    if (busy()) { setStatus("Bitte warten, bis der laufende Vorgang abgeschlossen ist", true); return; }
    m_source = m_original; clearRustHistory(); refresh(); setStatus("Entwurf verworfen"); }

void Studio::initializePreview() {
    QString error;
    m_previewReady = Preview::ensureRecoveryUnit(runtimeLauncher(), error);
    const QVariantMap current = Preview::state();
    const QString phase = current.value("phase").toString();
    if (m_previewReady && phase == "reverted" && !dirty() && !m_path.isEmpty()) open(m_path);
    if (phase == "pending" || phase == "testing") {
        m_previewActive = true;
        m_previewTimer.start();
        updatePreview();
    }
    if (!m_previewReady) setStatus("30 Sekunden Test nicht verfügbar: " + error, true);
    emit previewChanged(); emit changed();
}

bool Studio::cancelPreview() {
    if (!m_previewActive) return false;
    QString error;
    if (!Preview::cancel(error)) { setStatus(error, true); return false; }
    setStatus("Test wird beendet · gespeicherte Belegung wird wiederhergestellt");
    return true;
}

void Studio::updatePreview() {
    const QVariantMap current = Preview::state();
    const QString phase = current.value("phase").toString();
    if (phase == "pending" || phase == "testing") {
        m_previewActive = true;
        const qint64 deadline = current.value("deadline").toLongLong();
        m_previewRemaining = deadline > 0 ? qMax(0, static_cast<int>((deadline - QDateTime::currentMSecsSinceEpoch() + 999) / 1000)) : 30;
        const qint64 pid = current.value("pid").toLongLong();
        if (phase == "testing" && pid > 0 && ::kill(static_cast<pid_t>(pid), 0) != 0 && !m_previewRecoveryStarted) {
            m_previewRecoveryStarted = QProcess::startDetached(runtimeLauncher(), {"--recover-preview"});
            if (m_previewRecoveryStarted) setStatus("Testprozess wurde unterbrochen · Wiederherstellung läuft", true);
        }
        emit previewChanged(); emit changed(); return;
    }
    if (!m_previewActive) return;
    m_previewActive = false; m_previewRecoveryStarted = false; m_previewRemaining = 0; m_previewTimer.stop(); emit previewChanged();
    if (phase == "reverted") setStatus("Test beendet · gespeicherte Belegung wieder aktiv");
    else setStatus("Testfehler: " + current.value("message").toString(), true);
}

QString Studio::runtimeLauncher() const {
    return qEnvironmentVariable("KANATA_STUDIO_LAUNCHER", QCoreApplication::applicationFilePath());
}
