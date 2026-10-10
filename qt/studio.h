#pragma once

#include <QObject>
#include <QVariantList>
#include <QVariantMap>
#include <QByteArray>
#include <QString>
#include <QVector>
#include <QTimer>
#include <QStringList>
#include <QJsonObject>
#include <QFutureWatcher>
#include <QHash>

class Studio : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString path READ path NOTIFY documentChanged)
    Q_PROPERTY(QString status READ status NOTIFY statusChanged)
    Q_PROPERTY(QString source READ source NOTIFY documentChanged)
    Q_PROPERTY(bool dirty READ dirty NOTIFY documentChanged)
    Q_PROPERTY(bool canUndo READ canUndo NOTIFY historyChanged)
    Q_PROPERTY(bool canRedo READ canRedo NOTIFY historyChanged)
    Q_PROPERTY(QVariantList layers READ layers NOTIFY layersChanged)
    Q_PROPERTY(QVariantList keys READ keys NOTIFY keysChanged)
    Q_PROPERTY(QVariantMap keyNames READ keyNames NOTIFY keyNamesChanged)
    Q_PROPERTY(int selectedLayer READ selectedLayer NOTIFY selectionChanged)
    Q_PROPERTY(int selectedKey READ selectedKey NOTIFY selectionChanged)
    Q_PROPERTY(QString selectedPhysicalKey READ selectedPhysicalKey NOTIFY selectionChanged)
    Q_PROPERTY(QVariantMap selectedKeyState READ selectedKeyState NOTIFY actionsChanged)
    Q_PROPERTY(QVariantList keyStates READ keyStates NOTIFY actionsChanged)
    Q_PROPERTY(QString action READ action NOTIFY actionsChanged)
    Q_PROPERTY(QVariantList actionLabels READ actionLabels NOTIFY actionsChanged)
    Q_PROPERTY(QVariantMap theme READ theme NOTIFY themeChanged)
    Q_PROPERTY(QVariantList configForms READ configForms NOTIFY configFormsChanged)
    Q_PROPERTY(QVariantMap actionDetails READ actionDetails NOTIFY actionsChanged)
    Q_PROPERTY(QVariantMap layoutInfo READ layoutInfo NOTIFY layoutChanged)
    Q_PROPERTY(bool previewActive READ previewActive NOTIFY previewChanged)
    Q_PROPERTY(bool previewReady READ previewReady NOTIFY previewChanged)
    Q_PROPERTY(int previewRemaining READ previewRemaining NOTIFY previewChanged)
public:
    explicit Studio(QObject *parent = nullptr);
    ~Studio() override;
    QString path() const { return m_path; }
    QString status() const { return m_status; }
    QString source() const { return m_source; }
    bool dirty() const { return m_source != m_original; }
    bool canUndo() const { return m_canUndo; }
    bool canRedo() const { return m_canRedo; }
    QVariantList layers() const { return m_layers; }
    QVariantList keys() const { return m_keys; }
    QVariantMap keyNames() const { return m_keyNamesMap; }
    int selectedLayer() const { return m_selectedLayer; }
    int selectedKey() const { return m_selectedKey; }
    QString selectedPhysicalKey() const { return m_selectedPhysicalKey; }
    QVariantMap selectedKeyState() const { return m_selectedKeyState; }
    QVariantList keyStates() const { return m_keyStates; }
    QString action() const;
    QVariantList actionLabels() const { return m_actionLabels; }
    QVariantMap theme() const { return m_theme; }
    QVariantList configForms() const;
    QVariantMap actionDetails() const;
    QVariantMap layoutInfo() const { return m_layoutInfo; }
    bool previewActive() const { return m_previewActive; }
    bool previewReady() const { return m_previewReady; }
    int previewRemaining() const { return m_previewRemaining; }
    void initializePreview();
    Q_INVOKABLE bool open(const QString &filename);
    Q_INVOKABLE void selectLayer(int index);
    Q_INVOKABLE void selectKey(int index);
    Q_INVOKABLE bool selectPhysicalKey(const QString &name);
    Q_INVOKABLE QVariantMap keyStateFor(int index) const;
    Q_INVOKABLE QString actionFor(int index) const;
    Q_INVOKABLE QString displayKeyName(const QString &key) const;
    Q_INVOKABLE QString actionLabelFor(int index) const;
    Q_INVOKABLE bool setAction(const QString &value);
    Q_INVOKABLE bool addPhysicalKey(const QString &name, const QString &action = "_");
    Q_INVOKABLE bool setTapHold(const QString &head, int repeatMs, int holdMs, const QString &tap, const QString &hold);
    Q_INVOKABLE bool setMacro(const QString &head, const QVariantList &steps);
    Q_INVOKABLE bool addLayer(const QString &name);
    Q_INVOKABLE bool addLayer(const QString &name, int copyFrom);
    Q_INVOKABLE bool renameLayer(int index, const QString &name);
    Q_INVOKABLE bool deleteLayer(int index);
    Q_INVOKABLE bool replaceConfigForm(int formIndex, const QString &expression);
    Q_INVOKABLE bool addConfigForm(const QString &expression);
    Q_INVOKABLE void setLayoutChoice(const QString &choice);
    Q_INVOKABLE bool undo();
    Q_INVOKABLE bool redo();
    Q_INVOKABLE bool check();
    Q_INVOKABLE bool save();
    Q_INVOKABLE bool startPreview();
    Q_INVOKABLE bool cancelPreview();
    Q_INVOKABLE void discard();
    Q_INVOKABLE void report(const QString &message);
signals:
    void changed();
    void documentChanged();
    void keysChanged();
    void keyNamesChanged();
    void layersChanged();
    void configFormsChanged();
    void selectionChanged();
    void actionsChanged();
    void historyChanged();
    void themeChanged();
    void layoutChanged();
    void statusChanged();
    void previewChanged();
private:
    bool refresh(QString *error = nullptr);
    void applyView(const QJsonObject &view);
    void applyEnvironment(const QJsonObject &result);
    QString effectiveLayout() const;
    QString runtimeLauncher() const;
    bool serviceUsesOpenFile(QString &error) const;
    void updatePreview();
    void detectKeyboardLayout();
    void setStatus(const QString &message);
    QString kanataBinary() const;
    QString m_path;
    QString m_source;
    QString m_viewSource;
    QString m_original;
    void *m_rustEditor;
    bool m_canUndo = false;
    bool m_canRedo = false;
    QJsonObject rustRequest(QJsonObject request) const;
    bool rustEdit(const QString &operation, QJsonObject arguments = {});
    void clearRustHistory();
    QByteArray m_diskHash;
    QString m_status;
    QVariantList m_layers;
    QVariantList m_keys;
    QVariantMap m_theme;
    QVariantMap m_layoutInfo;
    QVariantList m_actions;
    QVariantList m_actionLabels;
    QVariantList m_configForms;
    QVariantMap m_actionDetails;
    mutable QHash<QString, QString> m_keyNames;
    QVariantMap m_keyNamesMap;
    QFutureWatcher<QJsonObject> m_environmentWatcher;
    int m_selectedLayer = 0;
    int m_selectedKey = -1;
    QString m_selectedPhysicalKey;
    QVariantMap m_selectedKeyState;
    QVariantList m_keyStates;
    bool m_previewReady = false;
    bool m_previewActive = false;
    bool m_previewRecoveryStarted = false;
    int m_previewRemaining = 0;
    QTimer m_previewTimer;
    QTimer m_environmentTimer;
};
