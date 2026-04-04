{ lib
, rustPlatform
, pkg-config
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "nina";
  version = "0.3.0";

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];

  postInstall = ''
    install -Dm644 completions/nina.bash $out/share/bash-completion/completions/nina
    install -Dm644 completions/_nina $out/share/zsh/site-functions/_nina
    install -Dm644 completions/nina.fish $out/share/fish/vendor_completions.d/nina.fish
  '';

  meta = with lib; {
    description = "NixOS Intuitive Navigation Assistant";
    homepage = "https://github.com/asha-software/nina";
    license = licenses.mit;
    mainProgram = "nina";
    platforms = platforms.linux ++ platforms.darwin;
  };
}
