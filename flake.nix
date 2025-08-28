{
  description = "Test environment for git-worktree-superproject";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { nixpkgs, ... }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          python = pkgs.python312;
          
          # Build pytest-result-bar from git source
          pytest-result-bar = python.pkgs.buildPythonPackage rec {
            pname = "pytest-result-bar";
            version = "main";
            pyproject = true;
            
            src = pkgs.fetchFromGitHub {
              owner = "timblaktu";
              repo = "pytest-result-bar";
              rev = "main";
              sha256 = "sha256-uwXjbyExT8w0SjqeiC0Afa64vA5AuCbGOhqGQgN2m+4=";
            };
            
            build-system = with python.pkgs; [ hatchling ];
            dependencies = with python.pkgs; [ pytest ];
            
            doCheck = false;  # Skip tests for this dependency
          };
          
          testEnv = python.withPackages (ps: with ps; [
            pytest
            pytest-cov
            pytest-mock
            pytest-timeout
            pytest-xdist
            pytest-result-bar
          ]);
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              testEnv
              pkgs.git
              pkgs.bash
            ];
          };
        });
    };
}