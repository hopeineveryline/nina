{ pkgs, nina, src }:

pkgs.testers.runNixOSTest {
  name = "nina-vm-smoke";

  nodes.machine = { pkgs, ... }: {
    nix.settings.experimental-features = [ "nix-command" "flakes" ];
    environment.systemPackages = [
      nina
      pkgs.cargo
      pkgs.git
      pkgs.openssl
      pkgs.pkg-config
      pkgs.rustc
    ];
  };

  testScript = ''
    start_all()
    machine.wait_for_unit("multi-user.target")
    machine.succeed("HOME=/root nina help")
    machine.succeed("HOME=/root nina hello")
    machine.succeed("rm -rf /root/nina-workspace && cp -r ${src} /root/nina-workspace && chmod -R u+w /root/nina-workspace")
    machine.succeed("cd /root/nina-workspace && HOME=/root nix develop .#default -c cargo test -- --list | tee /tmp/nina-tests.txt")
    machine.succeed("grep -q '150 tests' /tmp/nina-tests.txt")
  '';
}
