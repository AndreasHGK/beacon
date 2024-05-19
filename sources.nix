# Filtered source used for packages.
# Reduces the amount of rebuilds needed, as it tries to detect
# changes in any of the resulting files.
{ lib, craneLib, }: {
  # Keeps:
  # - All rust related files
  # - All `.js` files
  # - All `.css` and `.scss` files
  # - `panel/assets`
  # - `.sqlx/`
  # - `migrations/`
  rust = let
    filter = path: type:
      (builtins.match ".*/frontend/.*" path == null)
      && ((craneLib.filterCargoSources path type)
        || (builtins.match ".*\\.js$" path != null)
        || (builtins.match ".*\\.s?css$" path != null)
        || (builtins.match ".*/panel/assets/.*" path != null)
        || (builtins.match ".*/.sqlx/.*" path != null)
        || (builtins.match ".*/migrations/.*" path != null));
  in lib.cleanSourceWith {
    src = craneLib.path ./.;
    inherit filter;
  };
  # Keeps (in `./frontend`):
  # - All `.js/.ts/.jsx/.tsx` files
  # - All `.ico` files
  # - All `.css` files
  # - All `.json` files
  # - All `.mjs` files
  # - All `.ttf` files
  # - `yarn.lock`
  frontend = lib.cleanSourceWith {
    src = craneLib.path ./frontend;
    filter = path: type:
      (type == "directory") || (builtins.match ".*\\.[jt]sx?$" path != null)
      || (lib.hasSuffix ".ico" path) || (lib.hasSuffix ".css" path)
      || (lib.hasSuffix ".json" path) || (lib.hasSuffix ".mjs" path)
      || (lib.hasSuffix ".ttf" path) || (lib.hasSuffix "yarn.lock" path)
      || (lib.hasSuffix ".eslintignore" path);
  };
}
