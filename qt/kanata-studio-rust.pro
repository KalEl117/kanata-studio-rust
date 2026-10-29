QT += quick quickcontrols2 concurrent
CONFIG += c++17
SOURCES += main.cpp studio.cpp preview.cpp
HEADERS += studio.h preview.h rust_core.h
RESOURCES += resources.qrc
TARGET = kanata-studio-rust
unix {
    LIBS += $$PWD/../target/release/libkanata_studio_core.a -ldl -lpthread -lm
    PRE_TARGETDEPS += $$PWD/../target/release/libkanata_studio_core.a
}
win32 {
    # MSVC: the system libraries `--print native-static-libs` lists for the Rust core.
    LIBS += $$PWD/../target/release/kanata_studio_core.lib \
        -lkernel32 -lntdll -luserenv -lws2_32 -ldbghelp -ladvapi32 -luser32
    PRE_TARGETDEPS += $$PWD/../target/release/kanata_studio_core.lib
}
