{ lib
, rustPlatform
, fetchFromGitHub
, pkg-config
, openssl
, git
, stdenv
, darwin
, makeWrapper
, perl
}:

rustPlatform.buildRustPackage rec {
  pname = "workspace-manager";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
    makeWrapper
    perl # Required for building OpenSSL from source
  ];

  buildInputs = [
    openssl
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  # The binary is called 'workspace' from the workspace-manager package
  cargoBuildFlags = [ "--package" "workspace-manager" ];

  # Ensure git is available at runtime
  postInstall = ''
    wrapProgram $out/bin/workspace \
      --prefix PATH : ${lib.makeBinPath [ git ]}
  '';

  meta = with lib; {
    description = "Multi-repository management with Git worktrees";
    longDescription = ''
      A high-performance Rust tool for managing multiple git repositories as a cohesive workspace.
      Uses git worktrees to efficiently manage isolated workspaces with shared git objects.
    '';
    homepage = "https://github.com/timblaktu/git-worktree-superproject";
    license = licenses.mit;
    maintainers = [ ];
    mainProgram = "workspace";
    platforms = platforms.all;
  };
}
