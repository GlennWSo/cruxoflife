{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    overlays = [(import rust-overlay)];
    pkgs = import nixpkgs {
      inherit system overlays;
      config.allowUnfree = true;
      config.android_sdk.accept_license = true;
    };
    toolchain = pkgs.rust-bin.nightly.latest.default.override {
      extensions = ["rust-src"];
      targets = ["aarch64-linux-android" "armv7-linux-androideabi" "i686-linux-android" "x86_64-linux-android" "wasm32-unknown-unknown"];
    };

    android = pkgs.androidenv.composeAndroidPackages {
      # cmdLineToolsVersion = "8.0";
      # toolsVersion = "26.1.1";
      # platformToolsVersion = "34.0.5";
      buildToolsVersions = ["34.0.0"];
      includeEmulator = true;
      # emulatorVersion = "30.3.4";
      platformVersions = ["31" "32" "33" "34"];
      includeSources = true;
      includeSystemImages = true;
      systemImageTypes = ["google_apis_playstore"];
      # abiVersions = ["armeabi-v7a" "arm64-v8a"];
      # cmakeVersions = ["3.10.2"];
      includeNDK = true;
      ndkVersions = ["27.0.12077973"];
      # useGoogleAPIs = false;
      # useGoogleTVAddOns = false;
      # includeExtras = [
      #   "extras;google;gcm"
      # ];
    };
    avdmanagerBin = "${android.androidsdk}/bin/avdmanager";
    deviceTemplate = "5";
    mkAVD = pkgs.writeShellScriptBin "mkavd" ''
      ${avdmanagerBin} create avd -n tester -k 'system-images;android-34-ext12;google_apis_playstore;x86_64' -d ${deviceTemplate} $@
    '';
    emulatorBin = "${android.androidsdk}/bin/emulator";
    adbBin = "${android.androidsdk}/libexec/android-sdk/platform-tools/adb";
    findPort = pkgs.writeShellScriptBin "findport" ''
      for i in $(seq 5554 2 5584)
      do
          if [ -z "$(${adbBin} devices | grep emulator-$i)" ]
          then
              port=$i
              break
          fi
      done

      if [ -z "$port" ]
      then
          echo "Unfortunately, the emulator port space is exhausted!" >&2
          exit 1
      else
          # echo "We have a free TCP port: $port" >&2
          echo $port
      fi
    '';
    startAVD = pkgs.writeShellScriptBin "startavd" ''
      # set to -port to some int in (5554..=5584).step(2)
      ${emulatorBin} -avd tester -no-boot-anim $@
    '';
    installApk = pkgs.writeShellScriptBin "installapk" ''
      echo installs apk to avd
        ${adbBin} -s emulator-$1 install "$2"
    '';
    startApk = pkgs.writeShellScriptBin "startapk" ''
      echo starts apk on device
        ${adbBin} -s emulator-$1 shell am start -n com.glennwso.cruxoflife/com.glennwso.cruxoflife.MainActivity
    '';
    oneUp = pkgs.writeShellScriptBin "oneup" ''
      ${mkAVD}/bin/mkavd
      port="$(${findPort}/bin/findport)"
      echo port is $port
      ${startAVD}/bin/startavd -port $port

      ${adbBin} -s emulator-$port wait-for-device
        echo "Device state has been reached" >&2

        while ! ${adbBin} -s emulator-$port shell getprop dev.bootcomplete |  grep -q 1
        do
            sleep 5
        done

        echo "dev.bootcomplete property is 1" >&2
        echo "ready" >&2
        ${installApk}/bin/installapk $port $1
        delay 2
        ${startApk}/bin/startapk $port
    '';
    sign = pkgs.writeShellScriptBin "sign" ''
      jarsigner -verbose -sigalg SHA256withRSA -keystore .secrets/androidkeystore.jks -signedjar cruxlife_signed.aab app/build/outputs/bundle/release/app-release.aab key0
    '';
    # android = pkgs.androidenv.androidPkgs;
  in {
    packages.x86_64-linux.default = pkgs.hello;
    # packages.x86_64-linux.sdk = android.androidsdk;
    devShells.x86_64-linux.default = pkgs.mkShell rec {
      SDK = android.androidsdk;
      # JAVA_HOME = pkgs.jdk21;
      ANDROID_HOME = "${android.androidsdk}/libexec/android-sdk";
      ANDROID_NDK_ROOT = "${ANDROID_HOME}/ndk-bundle";
      GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${android.androidsdk}/libexec/android-sdk/build-tools/34.0.0/aapt2";
      RUSTC = "${toolchain}/bin/rustc";
      RUST_CARGO = "${toolchain}/bin/cargo";
      buildInputs = with pkgs; [
        typos
        pandoc

        mkAVD
        findPort
        startAVD
        installApk
        startApk
        oneUp
        sign

        toolchain
        rust-analyzer
        bacon
        cargo-insta

        cargo-ndk
        cargo-apk
        glib
        wasm-pack
        pnpm
        trunk
        nodejs
        static-server

        (android-studio.withSdk android.androidsdk)
        ktlint
        kotlin-language-server
        # gradle_8
        jdk21
        usbutils
        python312
      ];
    };
  };
}
