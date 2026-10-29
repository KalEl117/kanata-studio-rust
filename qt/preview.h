#pragma once

#include <QByteArray>
#include <QString>
#include <QVariantMap>

namespace Preview {
QString directory();
QVariantMap state();
bool ensureRecoveryUnit(const QString &binary, QString &error);
bool start(const QString &binary, const QString &configPath, const QByteArray &original,
           const QByteArray &candidate, QString &error, int durationMs = 30000);
bool cancel(QString &error);
int runWorker(bool recover);
bool processAlive(qint64 pid);
}
