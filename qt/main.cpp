#include "studio.h"
#include "preview.h"
#include <QCoreApplication>
#include <QGuiApplication>
#include <QDir>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QFile>
#include <QFileInfo>
#include <QFont>
#include <QJsonDocument>
#include <QJsonArray>
#include <QJsonObject>
#include <QScreen>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickWindow>
#include <QQuickItem>
#include <QTimer>
#include <QTemporaryDir>
#include <QThread>
#include <QUrl>
#include <cstdio>
#include <algorithm>

static QObject *findVisualObject(QObject *object, const QString &name) {
    if (object->objectName() == name) return object;
    if (auto *window = qobject_cast<QQuickWindow *>(object)) return findVisualObject(window->contentItem(), name);
    if (auto *item = qobject_cast<QQuickItem *>(object)) {
        for (QQuickItem *child : item->childItems()) if (QObject *found = findVisualObject(child, name)) return found;
    }
    return nullptr;
}

int main(int argc, char *argv[]) {
    QElapsedTimer startupTimer;
    startupTimer.start();
    const bool profileStartup = qEnvironmentVariableIsSet("KANATA_STUDIO_STARTUP_PROFILE");
    auto startupMark = [&](const char *stage) {
        if (profileStartup) std::fprintf(stderr, "startup %s: %lld ms\n", stage, static_cast<long long>(startupTimer.elapsed()));
    };
    if (argc > 1) {
        const QString mode = QString::fromLocal8Bit(argv[1]);
        if (mode == "--preview-worker" || mode == "--recover-preview") {
            QCoreApplication app(argc, argv);
            return Preview::runWorker(mode == "--recover-preview");
        }
    }
    QGuiApplication app(argc, argv);
    startupMark("Qt bereit");
    if (argc > 1 && QString::fromLocal8Bit(argv[1]) == "--display-check") {
        QJsonArray screens;
        for (const auto *screen : app.screens()) {
            const auto geometry = screen->geometry();
            screens.append(QJsonObject{{"name", screen->name()},
                {"devicePixelRatio", screen->devicePixelRatio()},
                {"width", geometry.width()}, {"height", geometry.height()}});
        }
        const QJsonObject result{{"platform", app.platformName()}, {"screens", screens}};
        std::puts(QJsonDocument(result).toJson(QJsonDocument::Compact).constData());
        return 0;
    }
    app.setApplicationName("Kanata Studio Rust");
    app.setFont(QFont("Iosevka Nerd Font", 10));
    Studio studio;
    startupMark("Konfiguration und Layout bereit");
    QString screenshot;
    QString screenshotView;
    QString languageOverride;
    bool checkOnly = false;
    bool selfTest = false;
    bool startupCheck = false;
    bool benchmarkUi = false;
    bool uiCheck = false;
    for (int i = 1; i < argc; ++i) {
        const QString arg = QString::fromLocal8Bit(argv[i]);
        if (arg == "--screenshot" && i + 1 < argc) screenshot = QString::fromLocal8Bit(argv[++i]);
        else if (arg == "--screenshot-view" && i + 1 < argc) screenshotView = QString::fromLocal8Bit(argv[++i]);
        else if (arg == "--language" && i + 1 < argc) languageOverride = QString::fromLocal8Bit(argv[++i]);
        else if (arg == "--check") checkOnly = true;
        else if (arg == "--self-test") selfTest = true;
        else if (arg == "--startup-check") startupCheck = true;
        else if (arg == "--benchmark-ui") benchmarkUi = true;
        else if (arg == "--ui-check") uiCheck = true;
        else studio.open(arg);
    }
    if (selfTest) {
        const int originalLayerCount = studio.layers().size();
        const QString originalSource = studio.source();
        const bool labelsReadable = studio.displayKeyName("caps") == "Feststelltaste"
            && studio.displayKeyName("ArrowUp") == "Pfeil hoch";
        const bool lazySelected = studio.selectPhysicalKey("F2") && studio.source() == originalSource
            && !studio.dirty() && !studio.canUndo() && studio.selectedKey() == -1;
        const bool lazyAssigned = lazySelected && studio.setAction("Minus") && studio.dirty()
            && studio.selectedKeyState().value("kind") == "own";
        const QString lazySource = studio.source();
        const bool lazyHistory = lazyAssigned && studio.undo() && studio.source() == originalSource
            && !studio.canUndo() && studio.selectedKey() == -1 && studio.selectedPhysicalKey() == "F2"
            && studio.redo() && studio.source() == lazySource && studio.selectedKey() >= 0;
        studio.discard(); studio.selectKey(0);
        const bool edited = !studio.keys().isEmpty() && studio.setAction("_") && studio.dirty();
        const bool editHistory = edited && studio.undo() && studio.source() == originalSource
            && studio.redo() && studio.source() != originalSource && studio.canUndo() && studio.canRedo() == false;
        const bool layerAdded = studio.addLayer("qt_test_layer") && studio.layers().size() == originalLayerCount + 1;
        const bool layerHistory = layerAdded && studio.undo() && studio.layers().size() == originalLayerCount
            && studio.redo() && studio.layers().size() == originalLayerCount + 1;
        const QString copiedAction = studio.actionFor(0);
        const bool layerCopied = layerHistory && studio.addLayer("qt_test_copy", studio.selectedLayer())
            && studio.layers().size() == originalLayerCount + 2 && studio.actionFor(0) == copiedAction;
        const bool layerRenamed = layerCopied && studio.renameLayer(studio.selectedLayer(), "qt_test_renamed")
            && studio.layers().last().toString() == "qt_test_renamed";
        const bool layerDeleted = layerRenamed && studio.deleteLayer(studio.selectedLayer())
            && studio.layers().size() == originalLayerCount + 1 && studio.undo()
            && studio.layers().size() == originalLayerCount + 2;
        const bool configAdded = studio.addConfigForm("(defalias qt_studio_test_alias a)");
        const QVariantMap addedConfig = configAdded ? studio.configForms().last().toMap() : QVariantMap{};
        const bool configEdited = configAdded && addedConfig.value("name").toString() == "defalias"
            && studio.replaceConfigForm(addedConfig.value("index").toInt(), "(defalias qt_studio_test_alias b)")
            && studio.source().contains("(defalias qt_studio_test_alias b)");
        const bool configHistory = configEdited && studio.undo()
            && studio.source().contains("(defalias qt_studio_test_alias a)")
            && studio.undo() && !studio.source().contains("qt_studio_test_alias");
        studio.discard();
        const bool restored = studio.source() == originalSource && !studio.dirty() && !studio.canUndo() && !studio.canRedo();
        const bool tapWorked = studio.actionDetails().value("kind") == "tap-hold"
            && studio.actionDetails().value("holdKind") == "modifiers"
            && studio.setTapHold("tap-hold", 180, 220, "esc", "lctl")
            && studio.source().contains("(tap-hold 180 220 esc lctl)")
            && studio.source().contains("(macro 5 (on-press tap-vkey relkana))");
        const bool macroWorked = studio.setMacro("macro", {"C-a", "120", "Digit1", "C-kp/", "C-kp*"})
            && studio.actionDetails().value("kind") == "macro"
            && studio.actionDetails().value("steps").toList().size() == 5;
        const int originalKeyCount = studio.keys().size();
        const bool physicalWorked = studio.addPhysicalKey("F1", "_") && studio.keys().size() == originalKeyCount + 1
            && studio.source().contains("C-kp/") && studio.check();
        const QString physicalStatus = studio.status();
        studio.discard();
        QTemporaryDir testDir;
        const QString testPath = testDir.path() + "/test.kbd";
        const QString binDir = testDir.path() + "/bin";
        QDir().mkpath(binDir);
        const QString fakeSystemctl = binDir + "/systemctl";
        QFile script(fakeSystemctl);
        bool fakeReady = script.open(QIODevice::WriteOnly);
        const QByteArray systemctlOk = "#!/bin/sh\nif [ \"$2\" = show ]; then printf 'argv[]=/usr/bin/kanata -c \"%s\" ; ignore_errors=no\\n' \"$KANATA_STUDIO_QT_TEST_CONFIG\"; fi\nexit 0\n";
        const QByteArray systemctlFail = "#!/bin/sh\nif [ \"$2\" = show ]; then printf 'argv[]=/usr/bin/kanata -c \"%s\" ; ignore_errors=no\\n' \"$KANATA_STUDIO_QT_TEST_CONFIG\"; exit 0; fi\nexit 1\n";
        if (fakeReady) { fakeReady = script.write(systemctlOk) == systemctlOk.size(); script.close(); fakeReady = fakeReady && script.setPermissions(QFile::ReadOwner | QFile::WriteOwner | QFile::ExeOwner); }
        const bool copied = QFile::copy(studio.path(), testPath);
        const QByteArray originalPath = qgetenv("PATH");
        qputenv("PATH", QFile::encodeName(binDir) + ':' + originalPath);
        qputenv("KANATA_STUDIO_QT_TEST_CONFIG", QFile::encodeName(testPath));
        Studio scratch;
        const bool saveWorked = fakeReady && copied && scratch.open(testPath) && scratch.setAction("_") && scratch.save() && !scratch.dirty();
        const QString saveStatus = scratch.status();
        const bool saveNotFlagged = !scratch.statusError();
        QFile savedFile(testPath);
        const bool savedMatches = savedFile.open(QIODevice::ReadOnly) && QString::fromUtf8(savedFile.readAll()) == scratch.source()
            && !scratch.selectedKeyState().value("changed").toBool();
        savedFile.close();
        if (fakeReady && script.open(QIODevice::WriteOnly | QIODevice::Truncate)) { script.write(systemctlFail); script.close(); }
        const QByteArray beforeRollback = scratch.source().toUtf8();
        const bool rollbackWorked = saveWorked && scratch.setAction("esc") && !scratch.save();
        const QString rollbackStatus = scratch.status();
        const bool rollbackFlagged = scratch.statusError();
        QFile afterFile(testPath);
        const bool rollbackMatches = afterFile.open(QIODevice::ReadOnly) && afterFile.readAll() == beforeRollback;
        afterFile.close();
        if (fakeReady && script.open(QIODevice::WriteOnly | QIODevice::Truncate)) { script.write(systemctlOk); script.close(); }
        qputenv("KANATA_STUDIO_QT_PREVIEW_DIR", QFile::encodeName(testDir.path() + "/preview"));
        QString previewError;
        const bool previewStarted = Preview::start(QCoreApplication::applicationFilePath(), testPath,
            beforeRollback, scratch.source().toUtf8(), previewError, 800);
        QElapsedTimer timer;
        timer.start();
        while (previewStarted && timer.elapsed() < 5000) {
            const QString phase = Preview::state().value("phase").toString();
            if (phase == "reverted" || phase == "error") break;
            QThread::msleep(80);
        }
        QFile previewFile(testPath);
        const bool previewWorked = previewStarted && Preview::state().value("phase") == "reverted"
            && previewFile.open(QIODevice::ReadOnly) && previewFile.readAll() == beforeRollback;
        previewFile.close();
        const bool cancelStarted = Preview::start(QCoreApplication::applicationFilePath(), testPath,
            beforeRollback, scratch.source().toUtf8(), previewError, 4000);
        timer.restart();
        bool candidateApplied = false;
        while (cancelStarted && timer.elapsed() < 2000) {
            QFile live(testPath);
            candidateApplied = live.open(QIODevice::ReadOnly) && live.readAll() == scratch.source().toUtf8();
            if (candidateApplied) break;
            QThread::msleep(50);
        }
        const bool cancelRequested = candidateApplied && Preview::cancel(previewError);
        timer.restart();
        while (cancelRequested && timer.elapsed() < 5000) {
            const QString phase = Preview::state().value("phase").toString();
            if (phase == "reverted" || phase == "error") break;
            QThread::msleep(80);
        }
        QFile canceledFile(testPath);
        const bool cancelWorked = cancelRequested && Preview::state().value("phase") == "reverted"
            && canceledFile.open(QIODevice::ReadOnly) && canceledFile.readAll() == beforeRollback;
        canceledFile.close();
        QFile candidateFile(testPath);
        const bool candidateWritten = candidateFile.open(QIODevice::WriteOnly | QIODevice::Truncate)
            && candidateFile.write(scratch.source().toUtf8()) == scratch.source().toUtf8().size();
        candidateFile.close();
        const QJsonObject request{{"configPath", testPath}, {"original", QString::fromLatin1(beforeRollback.toBase64())},
            {"candidate", QString::fromLatin1(scratch.source().toUtf8().toBase64())}};
        QFile requestFile(Preview::directory() + "/request.json");
        const bool requestWritten = requestFile.open(QIODevice::WriteOnly | QIODevice::Truncate)
            && requestFile.write(QJsonDocument(request).toJson()) > 0;
        requestFile.close();
        QFile stateFile(Preview::directory() + "/state.json");
        const bool stateWritten = stateFile.open(QIODevice::WriteOnly | QIODevice::Truncate)
            && stateFile.write("{\"phase\":\"testing\"}") > 0;
        stateFile.close();
        QFile recoveredFile(testPath);
        const bool recoveryWorked = candidateWritten && requestWritten && stateWritten && Preview::runWorker(true) == 0
            && Preview::state().value("phase") == "reverted"
            && recoveredFile.open(QIODevice::ReadOnly) && recoveredFile.readAll() == beforeRollback;
        recoveredFile.close();
        scratch.requestSave();
        const bool asyncStarted = scratch.busy();
        timer.restart();
        while (asyncStarted && (scratch.busy() || scratch.dirty()) && timer.elapsed() < 15000)
            QCoreApplication::processEvents(QEventLoop::AllEvents, 50);
        QFile asyncFile(testPath);
        const bool asyncSaveWorked = asyncStarted && !scratch.busy() && !scratch.dirty()
            && asyncFile.open(QIODevice::ReadOnly) && asyncFile.readAll() == scratch.source().toUtf8();
        asyncFile.close();
        const QByteArray asyncBytes = scratch.source().toUtf8();
        QFile bomFile(testPath);
        const bool bomWritten = bomFile.open(QIODevice::WriteOnly | QIODevice::Truncate)
            && bomFile.write("\xEF\xBB\xBF" + asyncBytes) == asyncBytes.size() + 3;
        bomFile.close();
        const bool bomSaved = bomWritten && scratch.open(testPath) && scratch.source().toUtf8() == asyncBytes
            && scratch.setAction("tab") && scratch.save();
        const bool bomKept = bomSaved && bomFile.open(QIODevice::ReadOnly)
            && bomFile.readAll() == "\xEF\xBB\xBF" + scratch.source().toUtf8() && !scratch.dirty();
        bomFile.close();
        const QString invalidPath = testDir.path() + "/invalid.kbd";
        QFile invalidFile(invalidPath);
        const bool invalidWritten = invalidFile.open(QIODevice::WriteOnly) && invalidFile.write("(defsrc \xFF a)\n(deflayer base a)\n") > 0;
        invalidFile.close();
        const bool invalidRejected = invalidWritten && !scratch.open(invalidPath) && scratch.path() == QFileInfo(testPath).absoluteFilePath();
        const bool encodingWorked = bomKept && invalidRejected;
        qunsetenv("KANATA_STUDIO_QT_PREVIEW_DIR");
        qunsetenv("KANATA_STUDIO_QT_TEST_CONFIG");
        qputenv("PATH", originalPath);
        const bool serviceParsing = Studio::serviceConfigPath("argv[]=/usr/bin/kanata -c /a.kbd ; x") == "/a.kbd"
            && Studio::serviceConfigPath("argv[]=/usr/bin/kanata --cfg /b.kbd ; x") == "/b.kbd"
            && Studio::serviceConfigPath("argv[]=/usr/bin/kanata --cfg=/c.kbd ; x") == "/c.kbd"
            && Studio::serviceConfigPath("argv[]=/usr/bin/kanata -c=/d.kbd -c /e.kbd ; x") == "/d.kbd"
            && Studio::serviceConfigPath("argv[]=/usr/bin/kanata --config \"/f g.kbd\" ; x") == "/f g.kbd"
            && Studio::serviceConfigPath("argv[]=/usr/bin/kanata --cfgx /h.kbd ; x").isEmpty()
            && Studio::serviceConfigPath("argv[]=/usr/bin/kanata ; x").isEmpty();
        const bool ok = serviceParsing && lazyHistory && labelsReadable && edited && editHistory && layerAdded && layerHistory && layerCopied && layerRenamed && layerDeleted && configAdded && configEdited && configHistory && restored && tapWorked && macroWorked && physicalWorked
            && saveWorked && savedMatches && rollbackWorked && rollbackMatches && previewWorked && cancelWorked && recoveryWorked && asyncSaveWorked && encodingWorked && saveNotFlagged && rollbackFlagged;
        std::fprintf(ok ? stdout : stderr, "%s\n", ok ? "Parser, Editoren, Speichern und Testmodus: OK" : "Selbsttest fehlgeschlagen");
        if (!ok) std::fprintf(stderr, "tap=%d macro=%d taste=%d save=%d rollback=%d preview=%d cancel=%d recover=%d async=%d service=%d encoding=%d flags=%d saveStatus=%s rollbackStatus=%s error=%s\n",
            tapWorked, macroWorked, physicalWorked, saveWorked, rollbackWorked && rollbackMatches, previewWorked,
            cancelWorked, recoveryWorked, asyncSaveWorked, serviceParsing, encodingWorked, saveNotFlagged && rollbackFlagged,
            saveStatus.toUtf8().constData(), rollbackStatus.toUtf8().constData(),
            previewError.toUtf8().constData());
        if (!ok) std::fprintf(stderr, "physicalStatus=%s\n", physicalStatus.toUtf8().constData());
        return ok ? 0 : 1;
    }
    if (checkOnly) {
        const bool ok = studio.check();
        std::fprintf(ok ? stdout : stderr, "%s\n", studio.status().toUtf8().constData());
        return ok ? 0 : 1;
    }
    if (screenshot.isEmpty() && !benchmarkUi && !uiCheck) studio.initializePreview();
    if (startupCheck) {
        const QJsonObject result{{"ok", studio.previewReady()}, {"source", studio.source()},
                                 {"state", QJsonObject::fromVariantMap(Preview::state())}, {"status", studio.status()}};
        const QByteArray output = QJsonDocument(result).toJson(QJsonDocument::Compact);
        std::fprintf(stdout, "%s\n", output.constData());
        return studio.previewReady() ? 0 : 1;
    }
    startupMark("Wiederherstellung bereit");
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("studio", &studio);
    engine.load(QUrl("qrc:/main.qml"));
    startupMark("Oberfläche geladen");
    if (engine.rootObjects().isEmpty()) return 1;
    if (uiCheck) {
        // Rebuilt combo models (here: a language switch) must not reset the key editor form,
        // otherwise "Apply" would silently write a different hold action. Draft only, never saved.
        QObject *root = engine.rootObjects().first();
        QObject *panel = root->findChild<QObject *>("editorPanel");
        QObject *holdMode = panel ? panel->findChild<QObject *>("holdMode") : nullptr;
        QObject *holdLayer = panel ? panel->findChild<QObject *>("holdLayerBox") : nullptr;
        const QString layer = studio.layers().value(0).toString();
        if (layer.isEmpty()) {
            std::fprintf(stderr, "UI-Prüfung: keine Konfiguration mit Ebenen geladen (Datei als Argument angeben)\n");
            return 1;
        }
        const QVariant language = root->property("uiLanguage");
        studio.selectLayer(0); studio.selectKey(0);
        bool ok = holdMode && holdLayer
            && studio.setTapHold("tap-hold", 150, 200, "esc", "(layer-while-held " + layer + ")");
        if (ok) QMetaObject::invokeMethod(panel, "sync");
        QCoreApplication::processEvents();
        const auto formShowsLayer = [&] {
            return holdMode->property("currentIndex").toInt() == 2 && holdLayer->property("currentText").toString() == layer;
        };
        const bool before = ok && formShowsLayer();
        root->setProperty("uiLanguage", language.toString() == "en" ? "de" : "en");
        QCoreApplication::processEvents();
        const bool after = ok && formShowsLayer();
        root->setProperty("uiLanguage", language);
        studio.discard();
        std::fprintf(before && after ? stdout : stderr, "UI-Prüfung: %s (vorher=%d nach Sprachwechsel=%d)\n",
                     before && after ? "OK" : "fehlgeschlagen", before, after);
        return before && after ? 0 : 1;
    }
    if (benchmarkUi) {
        QTimer::singleShot(200, &app, [&] {
            const QString original = studio.source();
            QObject *root = engine.rootObjects().first();
            QObject *panel = root->findChild<QObject *>("editorPanel");
            auto syncPanel = [&] { if (panel) QMetaObject::invokeMethod(panel, "sync"); };
            QString missing;
            for (int i = 1; i <= 24 && missing.isEmpty(); ++i) {
                const QString name = "F" + QString::number(i);
                bool exists = false;
                for (const QVariant &key : studio.keys()) if (key.toString().compare(name, Qt::CaseInsensitive) == 0) exists = true;
                if (!exists) missing = name;
            }
            bool lazyKeys = panel && !missing.isEmpty()
                && QMetaObject::invokeMethod(root, "choosePhysicalKey", Q_ARG(QVariant, QVariant(missing)))
                && studio.selectedPhysicalKey() == missing && studio.selectedKey() == -1
                && studio.source() == original && !studio.dirty() && !studio.canUndo();
            if (lazyKeys) {
                panel->setProperty("pickerTarget", "output");
                lazyKeys = QMetaObject::invokeMethod(panel, "addPickedKey", Q_ARG(QVariant, QVariant("Minus")))
                    && studio.action() == "Minus" && studio.dirty() && studio.selectedKey() >= 0
                    && studio.selectedKeyState().value("kind") == "own"
                    && studio.selectedKeyState().value("changed").toBool();
                QCoreApplication::processEvents();
                QObject *tile = findVisualObject(root, "keyTile_" + missing);
                QVariant comparisonText;
                lazyKeys = lazyKeys && tile && tile->property("draftChanged").toBool()
                    && QMetaObject::invokeMethod(root, "keyTooltip", Q_RETURN_ARG(QVariant, comparisonText), Q_ARG(QVariant, QVariant(studio.selectedKeyState())))
                    && comparisonText.toString().contains("→") && comparisonText.toString().contains(missing);
                const QString edited = studio.source();
                lazyKeys = lazyKeys && QMetaObject::invokeMethod(root, "undoEdit")
                    && studio.source() == original && !studio.dirty() && !studio.canUndo()
                    && studio.selectedPhysicalKey() == missing && studio.selectedKey() == -1;
                QCoreApplication::processEvents();
                tile = findVisualObject(root, "keyTile_" + missing);
                lazyKeys = lazyKeys && tile && !tile->property("draftChanged").toBool();
                lazyKeys = lazyKeys && QMetaObject::invokeMethod(root, "redoEdit") && studio.source() == edited
                    && studio.selectedPhysicalKey() == missing && studio.selectedKey() >= 0;
            }
            studio.discard(); syncPanel();
            bool pickerModifiers = panel;
            for (const QString &target : {QString("output"), QString("tap"), QString("chord"), QString("macro")}) {
                if (!pickerModifiers) break;
                pickerModifiers = QMetaObject::invokeMethod(panel, "openKeyPicker", Q_ARG(QVariant, QVariant(target)));
                QCoreApplication::processEvents();
                QObject *alt = panel->findChild<QObject *>("pickerAlt");
                QObject *super = panel->findChild<QObject *>("pickerSuper");
                QObject *modifiers = panel->findChild<QObject *>("pickerModifiers");
                pickerModifiers = pickerModifiers && alt && super && modifiers && modifiers->property("visible").toBool();
                if (!pickerModifiers) break;
                alt->setProperty("checked", true); super->setProperty("checked", true);
                QVariant output;
                pickerModifiers = QMetaObject::invokeMethod(panel, "pickedOutput", Q_RETURN_ARG(QVariant, output), Q_ARG(QVariant, QVariant("(multi lshft a)")))
                    && output.toString() == "(multi lalt lshft lmeta a)";
                if (target == "output" || target == "chord") {
                    pickerModifiers = pickerModifiers && QMetaObject::invokeMethod(panel, "addPickedKey", Q_ARG(QVariant, QVariant("a")))
                        && studio.action() == (target == "output" ? "(multi lalt lmeta a)" : "A-M-a");
                    studio.discard(); syncPanel();
                }
            }
            if (panel) QMetaObject::invokeMethod(panel, "openKeyPicker", Q_ARG(QVariant, QVariant("output")));
            studio.discard(); syncPanel();
            bool ok = lazyKeys && pickerModifiers && !studio.keys().isEmpty() && studio.addLayer("rust_benchmark_layer", 0);
            auto measure = [&](auto operation) {
                QVector<double> timings;
                for (int i = 0; i < 60 && ok; ++i) {
                    QElapsedTimer timer; timer.start();
                    operation(i);
                    timings.append(timer.nsecsElapsed() / 1000000.0);
                    QCoreApplication::processEvents();
                }
                std::sort(timings.begin(), timings.end());
                if (timings.isEmpty()) return QJsonObject{};
                return QJsonObject{{"medianMs", timings[timings.size()/2]},
                                   {"p95Ms", timings[int(timings.size()*0.95)]}, {"samples", timings.size()}};
            };
            const QJsonObject keys = measure([&](int i) { studio.selectKey(i % studio.keys().size()); syncPanel(); });
            const QJsonObject layers = measure([&](int i) { studio.selectLayer(i % studio.layers().size()); syncPanel(); });
            const QJsonObject macros = measure([&](int) {
                ok = studio.setMacro("macro", {"C-a", "120", "Digit1", "C-kp/", "C-kp*"});
                syncPanel();
                ok = ok && studio.actionLabelFor(studio.selectedKey()) == "Makro" && (!panel || panel->property("view").toInt() == 2);
                ok = ok && studio.undo(); syncPanel();
            });
            studio.discard();
            ok = ok && studio.source() == original && !studio.dirty();
            const QJsonObject result{{"ok", ok}, {"lazyKeyEditing", lazyKeys}, {"pickerModifiers", pickerModifiers}, {"keySelection", keys}, {"layerSelection", layers}, {"macroEditAndUndo", macros}};
            const QByteArray output = QJsonDocument(result).toJson(QJsonDocument::Compact);
            std::fprintf(stdout, "%s\n", output.constData());
            app.exit(ok ? 0 : 1);
        });
    }
    if (profileStartup) {
        if (auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
            QObject::connect(window, &QQuickWindow::frameSwapped, &app, [&, first = true]() mutable {
                if (first) { startupMark("Erstes Bild"); first = false; }
            });
        }
    }
    if (languageOverride == "de" || languageOverride == "en")
        engine.rootObjects().first()->setProperty("uiLanguage", languageOverride);
    if (!screenshot.isEmpty()) {
        const QVariant originalLayersVisible = engine.rootObjects().first()->property("layersVisible");
        const QVariant originalEditorVisible = engine.rootObjects().first()->property("editorVisible");
        if (screenshotView == "layers-hidden" || screenshotView == "panels-hidden")
            engine.rootObjects().first()->setProperty("layersVisible", false);
        if (screenshotView == "editor-hidden" || screenshotView == "panels-hidden")
            engine.rootObjects().first()->setProperty("editorVisible", false);
        if (screenshotView == "draft-comparison") {
            studio.selectLayer(0); studio.selectKey(0);
            studio.setTapHold("tap-hold", 150, 200, "esc", "(layer-switch special)");
            if (auto *panel = engine.rootObjects().first()->findChild<QObject *>("editorPanel")) QMetaObject::invokeMethod(panel, "sync");
            if (auto *tile = findVisualObject(engine.rootObjects().first(), "keyTile_CapsLock")) tile->setProperty("showComparison", true);
        }
        if (screenshotView == "second-layer") {
            studio.selectLayer(1); studio.selectKey(1);
            if (auto *panel = engine.rootObjects().first()->findChild<QObject *>("editorPanel")) QMetaObject::invokeMethod(panel, "sync");
        }
        if (screenshotView == "kanata-setup" || screenshotView == "kanata-setup-sample") {
            QObject *root = engine.rootObjects().first();
            if (auto *dialog = root->findChild<QObject *>("setupDialog")) QMetaObject::invokeMethod(dialog, "open");
            // A machine without permissions, so the command list can be checked on any desktop.
            if (screenshotView == "kanata-setup-sample")
                QTimer::singleShot(700, root, [root] {
                    root->setProperty("kanataStatus", QVariantMap{{"kanata", "/usr/bin/kanata"}, {"configPath", "/home/user/.config/kanata/kanata.kbd"},
                        {"service", false}, {"inputReadable", false}, {"uinputWritable", false}, {"relogin", true},
                        {"commands", QStringList{"sudo groupadd --system uinput", "sudo usermod -aG input,uinput \"$USER\"",
                            "echo 'KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"uinput\", OPTIONS+=\"static_node=uinput\"' | sudo tee /etc/udev/rules.d/99-input.rules",
                            "sudo udevadm control --reload-rules && sudo udevadm trigger"}}});
                });
        }
        if (screenshotView == "first-run-create") {
            if (auto *button = engine.rootObjects().first()->findChild<QObject *>("createConfigButton"))
                QMetaObject::invokeMethod(button, "clicked");
        }
        if (screenshotView == "config") {
            if (auto *dialog = engine.rootObjects().first()->findChild<QObject *>("configDialog"))
                QMetaObject::invokeMethod(dialog, "open");
        }
        if (screenshotView == "layer-dialog") {
            if (auto *dialog = engine.rootObjects().first()->findChild<QObject *>("addLayerDialog"))
                QMetaObject::invokeMethod(dialog, "open");
        }
        if (screenshotView == "rename-dialog") {
            if (auto *dialog = engine.rootObjects().first()->findChild<QObject *>("renameLayerDialog"))
                QMetaObject::invokeMethod(dialog, "open");
        }
        if (auto *panel = engine.rootObjects().first()->findChild<QObject *>("editorPanel")) {
            if (screenshotView == "macro") panel->setProperty("view", 2);
            else if (screenshotView == "tap-advanced") {
                panel->setProperty("view", 1);
                panel->setProperty("tapAdvanced", true);
            }
            else if (screenshotView == "action" || screenshotView == "action-advanced") {
                panel->setProperty("view", 0);
                panel->setProperty("actionAdvanced", screenshotView == "action-advanced");
            }
            else if (screenshotView == "action-catalog") {
                panel->setProperty("view", 0);
                if (auto *dialog = panel->findChild<QObject *>("actionChooser")) QMetaObject::invokeMethod(dialog, "open");
            }
            else if (screenshotView.endsWith("-picker") || screenshotView.startsWith("picker-")) {
                const bool macroPicker = screenshotView == "macro-picker";
                const bool chordPicker = screenshotView == "chord-picker";
                panel->setProperty("view", macroPicker ? 2 : chordPicker ? 0 : 1);
                QMetaObject::invokeMethod(panel, "openKeyPicker", Q_ARG(QVariant, QVariant(macroPicker ? "macro" : chordPicker ? "chord" : "tap")));
                if (screenshotView.startsWith("picker-")) panel->setProperty("pickerPage", screenshotView.mid(QString("picker-").size()));
            }
        }
        QTimer::singleShot(1200, &app, [&, originalLayersVisible, originalEditorVisible] {
            auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
            if (window) window->grabWindow().save(screenshot);
            engine.rootObjects().first()->setProperty("forceClose", true);
            if (screenshotView == "layers-hidden" || screenshotView == "panels-hidden")
                engine.rootObjects().first()->setProperty("layersVisible", originalLayersVisible);
            if (screenshotView == "editor-hidden" || screenshotView == "panels-hidden")
                engine.rootObjects().first()->setProperty("editorVisible", originalEditorVisible);
            app.quit();
        });
    }
    return app.exec();
}
