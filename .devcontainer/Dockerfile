FROM mcr.microsoft.com/devcontainers/rust:1

RUN curl -o ndk.zip https://dl.google.com/android/repository/android-ndk-r27d-linux.zip && \
    unzip ndk.zip && \
    mv android-ndk-r27d /ndk && \
    rm ndk.zip && \
    rustup target add aarch64-linux-android
