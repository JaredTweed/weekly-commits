set shell := ["bash", "-uc"]

bin_dir := env_var("HOME") / ".local/bin"
apps_dir := env_var("HOME") / ".local/share/applications"
desktop_file := apps_dir / "dev.funinkina.weekly-commits.cosmic.desktop"

build:
    cargo build --release

install: build
    mkdir -p "{{bin_dir}}" "{{apps_dir}}"
    install -m 755 target/release/cosmic-weekly-commits "{{bin_dir}}/"
    install -m 755 target/release/cosmic-weekly-commits-settings "{{bin_dir}}/"
    printf '%s\n' \
        '[Desktop Entry]' \
        'Name=Weekly Commits' \
        'Comment=Show weekly git contributions in the COSMIC panel' \
        'Exec={{bin_dir}}/cosmic-weekly-commits' \
        'Terminal=false' \
        'Type=Application' \
        'StartupNotify=true' \
        'NotShowIn=X-COSMIC' \
        'Categories=Development;' \
        'Keywords=COSMIC;Applet;Git;Commits;GitHub;GitLab;Gitea;' \
        'X-CosmicApplet=true' \
        'X-CosmicHoverPopup=None' \
        > "{{desktop_file}}"
    update-desktop-database "{{apps_dir}}" 2>/dev/null || true

uninstall:
    rm -f "{{bin_dir}}/cosmic-weekly-commits" "{{bin_dir}}/cosmic-weekly-commits-settings" "{{desktop_file}}"
    update-desktop-database "{{apps_dir}}" 2>/dev/null || true
