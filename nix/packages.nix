{
  perSystem =
    { self', ... }:
    {
      packages = {
        inherit (self'.checks) smux;
        default = self'.checks.smux;
      };
    };
}
