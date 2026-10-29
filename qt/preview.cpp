#include "preview.h"
#include "rust_core.h"

#include <QCoreApplication>
#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QProcess>
#include <QProcessEnvironment>
#include <QSaveFile>
#ifdef Q_OS_WIN
#include <QSettings>
#ifndef NOMINMAX
#define NOMINMAX
#endif
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#else
#include <signal.h>
#endif

namespace {
QString fileName(const QString &name) { return Preview::directory() + '/' + name; }
bool secureDirectory() {
    if (!QDir().mkpath(Preview::directory())) return false;
#ifdef Q_OS_WIN
    // %LOCALAPPDATA% is already private to the user; Qt maps permissions only to the read-only flag.
    return true;
#endif
    return QFile::setPermissions(Preview::directory(), QFile::ReadOwner | QFile::WriteOwner | QFile::ExeOwner);
}
QJsonObject readJson(const QString &name) {
    QFile file(fileName(name));
    if (!file.open(QIODevice::ReadOnly)) return {};
    return QJsonDocument::fromJson(file.readAll()).object();
}
bool writeJson(const QString &name, const QJsonObject &object) {
    if (!secureDirectory()) return false;
    QSaveFile file(fileName(name));
    if (!file.open(QIODevice::WriteOnly)) return false;
    file.setPermissions(QFile::ReadOwner | QFile::WriteOwner);
    const QByteArray content = QJsonDocument(object).toJson(QJsonDocument::Compact);
    return file.write(content) == content.size() && file.commit();
}
QByteArray readBytes(const QString &path) {
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly)) return {};
    return file.readAll();
}
void finish(const QString &phase, const QString &message = {}) {
    QJsonObject status{{"phase", phase}};
    if (!message.isEmpty()) status.insert("message", message);
    writeJson("state.json", status);
    if (phase == "reverted") QFile::remove(fileName("request.json"));
    QFile::remove(fileName("cancel"));
}
#ifdef Q_OS_WIN
// Minutes since the epoch at boot, as the Rust worker records it; may differ by one minute.
QString bootId() { return QString::number((QDateTime::currentMSecsSinceEpoch() - static_cast<qint64>(GetTickCount64())) / 60000); }
bool sameBoot(const QString &recorded) { return qAbs(recorded.toLongLong() - bootId().toLongLong()) <= 1; }
#else
QString bootId() { return QString::fromUtf8(readBytes("/proc/sys/kernel/random/boot_id")).trimmed(); }
bool sameBoot(const QString &recorded) { return recorded == bootId(); }
#endif
}

namespace Preview {
QString directory() {
    const QString testDirectory = qEnvironmentVariable("KANATA_STUDIO_QT_PREVIEW_DIR");
    if (!testDirectory.isEmpty()) return testDirectory;
#ifdef Q_OS_WIN
    return QDir::fromNativeSeparators(qEnvironmentVariable("LOCALAPPDATA")) + "/kanata-studio-rust/preview";
#else
    return QDir::homePath() + "/.local/state/kanata-studio-rust/preview";
#endif
}

bool processAlive(qint64 pid) {
    if (pid <= 0) return false;
#ifdef Q_OS_WIN
    HANDLE process = OpenProcess(SYNCHRONIZE, FALSE, static_cast<DWORD>(pid));
    if (!process) return false;
    const bool alive = WaitForSingleObject(process, 0) == WAIT_TIMEOUT;
    CloseHandle(process);
    return alive;
#else
    return ::kill(static_cast<pid_t>(pid), 0) == 0;
#endif
}
QVariantMap state() { return readJson("state.json").toVariantMap(); }

bool ensureRecoveryUnit(const QString &binary, QString &error) {
    if (!secureDirectory()) { error = "Vorschauverzeichnis kann nicht geschützt werden"; return false; }
#ifdef Q_OS_WIN
    // The user's Run key restores an interrupted test at login, like the systemd unit on Linux.
    QSettings run("HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", QSettings::NativeFormat);
    const QString command = '"' + QDir::toNativeSeparators(binary) + "\" --recover-preview";
    if (run.value("KanataStudioRecover").toString() != command) {
        run.setValue("KanataStudioRecover", command);
        run.sync();
        if (run.status() != QSettings::NoError) { error = "Wiederherstellungsdienst kann nicht geschrieben werden"; return false; }
    }
#else
    const QString unitDir = qEnvironmentVariable("KANATA_STUDIO_UNIT_DIR", QDir::homePath() + "/.config/systemd/user");
    if (!QDir().mkpath(unitDir)) { error = "Wiederherstellungsdienst kann nicht angelegt werden"; return false; }
    const QString unitPath = unitDir + "/kanata-studio-rust-recover.service";
    // systemd expands % specifiers and C escapes even inside quotes, so the path must be escaped for both.
    QString execPath = binary;
    execPath.replace('\\', "\\\\").replace('"', "\\\"").replace('%', "%%");
    const QByteArray unit = ("[Unit]\nDescription=Restore interrupted Kanata Studio Rust preview\nBefore=kanata.service\n\n"
                             "[Service]\nType=oneshot\nExecStart=\"" + execPath + "\" --recover-preview\n\n"
                             "[Install]\nWantedBy=default.target\n").toUtf8();
    const bool unitChanged = readBytes(unitPath) != unit;
    const QString setupStamp = fileName("recovery-unit.ready");
    const bool setupPending = readBytes(setupStamp) != unit;
    if (unitChanged) {
        QSaveFile file(unitPath);
        if (!file.open(QIODevice::WriteOnly) || file.write(unit) != unit.size() || !file.commit()) {
            error = "Wiederherstellungsdienst kann nicht geschrieben werden"; return false;
        }
    }
    if (unitChanged || setupPending) {
        QProcess daemon;
        QProcessEnvironment native = QProcessEnvironment::systemEnvironment();
        if (native.contains("KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH")) native.insert("LD_LIBRARY_PATH", native.value("KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH"));
        daemon.setProcessEnvironment(native);
        daemon.start("systemctl", {"--user", "daemon-reload"});
        if (!daemon.waitForStarted(3000) || !daemon.waitForFinished(10000)
            || daemon.exitStatus() != QProcess::NormalExit || daemon.exitCode() != 0) {
            error = "systemd konnte nicht neu geladen werden"; return false;
        }
    }
    const QFileInfo enabledUnit(unitDir + "/default.target.wants/kanata-studio-rust-recover.service");
    if (enabledUnit.canonicalFilePath() != QFileInfo(unitPath).canonicalFilePath()) {
        QProcess enable;
        QProcessEnvironment native = QProcessEnvironment::systemEnvironment();
        if (native.contains("KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH")) native.insert("LD_LIBRARY_PATH", native.value("KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH"));
        enable.setProcessEnvironment(native);
        enable.start("systemctl", {"--user", "enable", "kanata-studio-rust-recover.service"});
        if (!enable.waitForStarted(3000) || !enable.waitForFinished(10000)
            || enable.exitStatus() != QProcess::NormalExit || enable.exitCode() != 0) {
            error = "Wiederherstellungsdienst konnte nicht aktiviert werden"; return false;
        }
    }
    if (unitChanged || setupPending) {
        QSaveFile stamp(setupStamp);
        if (!stamp.open(QIODevice::WriteOnly) || !stamp.setPermissions(QFile::ReadOwner | QFile::WriteOwner)
            || stamp.write(unit) != unit.size() || !stamp.commit()) {
            error = "Dienstbereitschaft konnte nicht gespeichert werden"; return false;
        }
    }
#endif
    const QJsonObject status = readJson("state.json");
    if (status.value("phase") == "testing") {
        if (sameBoot(status.value("bootId").toString()) && processAlive(static_cast<qint64>(status.value("pid").toDouble()))) return true;
    }
    if ((status.value("phase") == "testing" || status.value("phase") == "pending" || status.value("phase") == "error")
        && !readJson("request.json").isEmpty()) {
        QProcess process;
        process.start(binary, {"--recover-preview"});
        if (!process.waitForStarted(3000) || !process.waitForFinished(15000) || process.exitCode() != 0) {
            error = "Unterbrochener Test konnte nicht wiederhergestellt werden"; return false;
        }
    }
    return true;
}

bool start(const QString &binary, const QString &configPath, const QByteArray &original,
           const QByteArray &candidate, QString &error, int durationMs) {
    const QString phase = readJson("state.json").value("phase").toString();
    if (phase == "pending" || phase == "testing") { error = "Ein Test läuft bereits"; return false; }
    if (QFile::exists(fileName("request.json"))) { error = "Ein vorheriger Test muss zuerst wiederhergestellt werden"; return false; }
    if (QFileInfo(configPath).isSymLink()) { error = "Verknüpfte Konfigurationsdateien können nicht getestet werden"; return false; }
    const QJsonObject request{{"configPath", configPath}, {"original", QString::fromLatin1(original.toBase64())},
                              {"candidate", QString::fromLatin1(candidate.toBase64())},
                              {"deadline", QDateTime::currentMSecsSinceEpoch() + durationMs}};
    if (!writeJson("request.json", request) || !writeJson("state.json", {{"phase", "pending"}})) {
        QFile::remove(fileName("request.json"));
        error = "Testdaten konnten nicht geschützt gespeichert werden"; return false;
    }
    if (!QProcess::startDetached(binary, {"--preview-worker"})) {
        finish("error", "Testprozess konnte nicht gestartet werden");
        error = "Testprozess konnte nicht gestartet werden"; return false;
    }
    return true;
}

bool cancel(QString &error) {
    const QString phase = readJson("state.json").value("phase").toString();
    if (phase != "pending" && phase != "testing") { error = "Es läuft kein Test"; return false; }
    QFile file(fileName("cancel"));
    if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate)) { error = "Test konnte nicht beendet werden"; return false; }
    file.setPermissions(QFile::ReadOwner | QFile::WriteOwner);
    return true;
}

int runWorker(bool recover) {
    const QByteArray request = QJsonDocument(QJsonObject{{"op", "worker"}, {"directory", directory()},
                                                        {"recover", recover}}).toJson(QJsonDocument::Compact);
    void *editor = ks_editor_new();
    char *response = ks_editor_request(editor, request.constData());
    const bool ok = response && QJsonDocument::fromJson(QByteArray(response)).object().value("ok").toBool();
    ks_string_free(response);
    ks_editor_free(editor);
    return ok ? 0 : 1;
}
}
