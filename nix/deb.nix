{
  lib,
  stdenvNoCC,
  replaceVars,
  dpkg,
  binutils,
  whois42d-ng-static,
}:
let
  manifest = lib.importTOML ../Cargo.toml;
  pname = manifest.package.name;
  inherit (manifest.package) version;
  debVersion = "${version}-1";
  architecture =
    {
      x86_64-linux = "amd64";
      aarch64-linux = "arm64";
    }
    .${stdenvNoCC.hostPlatform.system};
  control = replaceVars ./debian/control {
    inherit pname architecture;
    version = debVersion;
    installedSize = null;
  };
in
stdenvNoCC.mkDerivation {
  pname = "${pname}-deb";
  inherit version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../LICENSE
      ../README.md
      ../resources/whois42d-ng.service
      ../resources/whois42d-ng.socket
      ../resources/whois42d-ng-rdap.socket
    ];
  };

  nativeBuildInputs = [ dpkg ];

  buildPhase = ''
    runHook preBuild
    install -Dm755 ${whois42d-ng-static}/bin/${pname} root/usr/bin/${pname}
    install -Dm644 LICENSE root/usr/share/doc/${pname}/copyright
    install -Dm644 README.md root/usr/share/doc/${pname}/README.md
    install -Dm644 ${whois42d-ng-static}/share/bash-completion/completions/${pname}.bash \
      root/usr/share/bash-completion/completions/${pname}
    install -Dm644 ${whois42d-ng-static}/share/zsh/site-functions/_${pname} \
      root/usr/share/zsh/vendor-completions/_${pname}
    install -Dm644 ${whois42d-ng-static}/share/fish/vendor_completions.d/${pname}.fish \
      root/usr/share/fish/vendor_completions.d/${pname}.fish
    install -Dm644 resources/whois42d-ng{,-rdap}.socket -t root/usr/lib/systemd/system
    substitute resources/whois42d-ng.service root/usr/lib/systemd/system/whois42d-ng.service \
      --replace-fail '/usr/local/bin' '/usr/bin'
    install -Dm644 ${control} root/DEBIAN/control
    install -m755 ${./debian/postinst} root/DEBIAN/postinst
    install -m755 ${./debian/prerm} root/DEBIAN/prerm
    install -m755 ${./debian/postrm} root/DEBIAN/postrm
    substituteInPlace root/DEBIAN/control \
      --replace-fail @installedSize@ "$(du -sk --exclude=DEBIAN root | cut -f1)"
    dpkg-deb --root-owner-group -Zxz --build root ${pname}_${debVersion}_${architecture}.deb
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm644 -t $out *.deb
    runHook postInstall
  '';

  doInstallCheck = true;
  nativeInstallCheckInputs = [ binutils ];
  installCheckPhase = ''
    runHook preInstallCheck
    deb="$out/${pname}_${debVersion}_${architecture}.deb"
    test "$(dpkg-deb -f "$deb" Architecture)" = '${architecture}'
    dpkg-deb -R "$deb" unpacked
    test -x unpacked/DEBIAN/postinst
    test -x unpacked/DEBIAN/prerm
    test -x unpacked/DEBIAN/postrm
    if readelf -l unpacked/usr/bin/${pname} | grep -q INTERP; then
      echo 'Debian binary must not require a dynamic loader' >&2
      exit 1
    fi
    if readelf -d unpacked/usr/bin/${pname} | grep -q NEEDED; then
      echo 'Debian binary must not depend on shared libraries' >&2
      exit 1
    fi
    test "$(grep '^ExecStart=' unpacked/usr/lib/systemd/system/whois42d-ng.service)" = \
      'ExecStart=/usr/bin/whois42d-ng --registry /var/lib/whois42d/registry'
    unpacked/usr/bin/${pname} --help > /dev/null
    runHook postInstallCheck
  '';
}
