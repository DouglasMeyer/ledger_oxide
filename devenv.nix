{
  pkgs,
  lib,
  config,
  ...
}:
{
  packages = with pkgs; [
    playwright-driver.browsers
    pkgs.postgresql
  ];

  env = {
    PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers}";
    PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = 1;
  };

  languages = {
    rust.enable = true;
    javascript = {
      enable = true;
      npm = {
        enable = true;
        install.enable = true;
      };
    };
  };

  services = {
    postgres = {
      enable = true;
      initialDatabases = [
        {
          name = "ledger_oxide";
        }
      ];
    };
  };
  processes = {
    backend = {
      exec = "cargo watch -x run";
      cwd = "${config.git.root}/api";
    };
    frontend = {
      exec = "npm run dev";
      cwd = "${config.git.root}/frontend";
    };
  };
}
