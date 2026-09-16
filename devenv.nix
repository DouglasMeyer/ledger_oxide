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
    DATABASE_URL = "postgres://localhost:5432/ledger_oxide";
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


  profiles = {
    testing.module = {
      env.DATABASE_URL = "postgres://localhost:5432/ledger_oxide_test";
      tasks = {
        "db:prepare" = {
          exec = ''
            createdb ledger_oxide_test
            cat backend/seeds/seed.sql | psql ledger_oxide_test
          '';
          after = [ "devenv:processes:postgres" ];
        };
      };
    };
  };
}
