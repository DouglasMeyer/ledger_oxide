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
    DATABASE_URL = if config.devenv.isTesting
      then "postgres://localhost:5432/ledger_oxide_test"
      else "postgres://localhost:5432/ledger_oxide";
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
      listen_addresses = "*";
      initialDatabases = [
        { name = "ledger_oxide"; }
      ];
    };
  };
  processes = {
    backend = {
      exec = "cargo run";
      cwd = "${config.git.root}/backend";
      ready.http.get = {
        port = 4000;
        path = "/health";
      };
      after = ["devenv:processes:postgres"];
    };
    frontend = {
      exec = "npm run dev";
      cwd = "${config.git.root}/frontend";
      after = ["devenv:processes:backend"];
    };
  };
  scripts.seed.exec = ''
    cat backend/seeds/seed.sql | psql ledger_oxide
  '';

  enterTest = ''
    wait_for_port 4000
    cat backend/seeds/seed.sql | psql ledger_oxide_test
    ./frontend/playwright_tests.sh
  '';

}
