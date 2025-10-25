# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  lib,
  pkg-config,
  rustPlatform,
  fetchFromGitHub,
  stdenv,
  apple-sdk,
  installShellFiles,
  installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  withNoDefaultFeatures ? false,
  withFeatures ? [ ],
}:

let
  version = "0.1.0";
  hash = "";
  cargoHash = "";
in

rustPlatform.buildRustPackage rec {
  inherit cargoHash version;

  pname = "calendula";

  src = fetchFromGitHub {
    inherit hash;
    owner = "pimalaya";
    repo = "calendula";
    rev = "v${version}";
  };

  buildNoDefaultFeatures = withNoDefaultFeatures;
  buildFeatures = withFeatures;

  nativeBuildInputs = [
    pkg-config
  ]
  ++ lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildInputs = lib.optional stdenv.hostPlatform.isDarwin apple-sdk;

  # unit tests only
  cargoTestFlags = [ "--lib" ];
  doCheck = false;
  auditable = false;

  postInstall = ''
    mkdir -p $out/share/{completions,man}
  ''
  + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    "$out"/bin/calendula man "$out"/share/man
  ''
  + lib.optionalString installManPages ''
    installManPage "$out"/share/man/*
  ''
  + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    "$out"/bin/calendula completion bash > "$out"/share/completions/calendula.bash
    "$out"/bin/calendula completion elvish > "$out"/share/completions/calendula.elvish
    "$out"/bin/calendula completion fish > "$out"/share/completions/calendula.fish
    "$out"/bin/calendula completion powershell > "$out"/share/completions/calendula.powershell
    "$out"/bin/calendula completion zsh > "$out"/share/completions/calendula.zsh
  ''
  + lib.optionalString installShellCompletions ''
    installShellCompletion "$out"/share/completions/calendula.{bash,fish,zsh}
  '';

  meta = {
    description = "CLI to manage calendars";
    mainProgram = "calendula";
    homepage = "https://github.com/pimalaya/calendula";
    changelog = "https://github.com/pimalaya/calendula/blob/v${version}/CHANGELOG.md";
    license = lib.licenses.agpl3Only;
    maintainers = with lib.maintainers; [
      soywod
    ];
  };
}
