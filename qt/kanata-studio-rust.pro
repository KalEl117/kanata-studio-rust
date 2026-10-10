QT += quick quickcontrols2 concurrent
CONFIG += c++17
SOURCES += main.cpp studio.cpp preview.cpp
HEADERS += studio.h preview.h rust_core.h
RESOURCES += resources.qrc
TARGET = kanata-studio-rust
LIBS += $$PWD/../target/release/libkanata_studio_core.a -ldl -lpthread -lm
PRE_TARGETDEPS += $$PWD/../target/release/libkanata_studio_core.a
