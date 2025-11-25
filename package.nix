{ lib
, rustPlatform
, fetchFromGitHub
, pkg-config
, openssl
, git
, libgit2
, stdenv
, darwin
, makeWrapper
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
  ];

  buildInputs = [
    openssl
    libgit2
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  # Use system libraries instead of vendored ones
  OPENSSL_NO_VENDOR = 1;
  LIBGIT2_NO_VENDOR = 1;

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
