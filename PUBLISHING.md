# PioPulse Package Manager Publishing Guide

This guide details how to compile, release, and distribute the **PioPulse Flasher TUI** to macOS, Linux, and Windows package managers: **Homebrew (Brew)**, **Scoop**, **Chocolatey (Choco)**, **WinGet**, and the **Arch User Repository (AUR)**.

---

## 🚀 Step 1: Push a New Release Tag (Automatic Binary Compilation)

A GitHub Actions workflow is set up at `.github/workflows/release.yml`. When you push a git tag starting with `v` (e.g., `v0.1.2`), the workflow automatically:
1. Compiles optimized binaries for Linux (x86_64), macOS (Intel + Apple Silicon), and Windows (x86_64).
2. Archives them (`.tar.gz` and `.zip`).
3. Computes the SHA256 checksums.
4. Creates a new GitHub Release draft/published release with all artifacts attached.

### How to trigger:
```bash
# 1. Update version in Cargo.toml (e.g., version = "0.1.2")
# 2. Commit changes
git add Cargo.toml
git commit -m "bump: release version v0.1.2"

# 3. Create and push tag
git tag v0.1.2
git push origin main --tags
```

Once the workflow finishes, go to your GitHub repository's **Releases** page to view the compiled binary URLs and download the `checksums.txt` file (or run `sha256sum <file>` to get the hashes).

---

## 🍺 1. Homebrew (macOS / Linux)

To distribute via Homebrew, you typically host a personal tap repository (e.g., `github.com/Wang-Yang-source/homebrew-tap`).

### Setup Steps:
1. Create a public repository named **`homebrew-tap`** on GitHub.
2. Inside that repository, create a folder named `Formula` if it doesn't exist.
3. Copy the template from [`packaging/brew/piopulse.rb`](file:///home/waya/Projects/PioPulse/packaging/brew/piopulse.rb) into `Formula/piopulse.rb`.
4. Update the version number and replace the placeholder `sha256` checksums with the actual hashes of the compiled release archives:
   - `piopulse-macos-aarch64.tar.gz`
   - `piopulse-macos-x86_64.tar.gz`
   - `piopulse-linux-x86_64.tar.gz`
5. Commit and push the formula changes to your `homebrew-tap` repository.

### User installation:
```bash
brew tap Wang-Yang-source/tap
brew install piopulse
```

---

## 🍨 2. Scoop (Windows Portable)

You can publish to a custom Scoop bucket (e.g., `github.com/Wang-Yang-source/scoop-bucket`) or submit to the official Scoop buckets.

### Setup Steps:
1. Create a public repository named **`scoop-bucket`** on GitHub.
2. Copy [`packaging/scoop/piopulse.json`](file:///home/waya/Projects/PioPulse/packaging/scoop/piopulse.json) to the repository root.
3. Update the `version` and insert the hash of `piopulse-windows-x86_64.zip`.
4. Commit and push.

### User installation:
```powershell
scoop bucket add Wang-Yang-bucket https://github.com/Wang-Yang-source/scoop-bucket.git
scoop install piopulse
```

---

## 🍫 3. Chocolatey (Windows Installer)

Chocolatey packages are packaged as `.nupkg` files and pushed to [chocolatey.org](https://community.chocolatey.org/).

### Setup Steps:
1. Install Chocolatey on your Windows build machine.
2. Open a terminal in [`packaging/choco/`](file:///home/waya/Projects/PioPulse/packaging/choco/).
3. Open `tools/chocolateyinstall.ps1` and replace the placeholder checksum with the SHA256 of `piopulse-windows-x86_64.zip`. Update the version in `piopulse.nuspec` if needed.
4. Package the files:
   ```powershell
   choco pack
   ```
   This will generate `piopulse.0.1.1.nupkg` (or the version specified).
5. Obtain your Chocolatey API Key from your account page on [chocolatey.org](https://community.chocolatey.org/).
6. Push the package:
   ```powershell
   choco apikey --key <YOUR_API_KEY> --source https://push.chocolatey.org/
   choco push piopulse.0.1.1.nupkg --source https://push.chocolatey.org/
   ```

### User installation:
```powershell
choco install piopulse
```

---

## 🪟 4. WinGet (Windows Package Manager)

WinGet manifests are submitted to the official [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) repository.

### Setup Steps:
You can automate submissions easily using the **`wingetcreate`** command-line tool.
1. Download `wingetcreate` from [GitHub Releases](https://github.com/microsoft/winget-create/releases).
2. Submit the package:
   ```powershell
   wingetcreate new https://github.com/Wang-Yang-source/piopulse/releases/download/v0.1.1/piopulse-windows-x86_64.zip
   ```
   This will prompt you for package info and automatically calculate the hash, generate the YAML manifest, and submit a Pull Request to `microsoft/winget-pkgs` using your GitHub account!

Alternatively, you can manually clone the `microsoft/winget-pkgs` repository and add the manifest generated in [`packaging/winget/piopulse.yaml`](file:///home/waya/Projects/PioPulse/packaging/winget/piopulse.yaml).

### User installation:
```powershell
winget install Wang-Yang.PioPulse
```

---

## ❄️ 5. AUR - Arch User Repository (Arch Linux)

AUR packages are hosted as git repositories managed by the Arch Linux team.

### Setup Steps:
1. Create an AUR account at [aur.archlinux.org](https://aur.archlinux.org/).
2. Upload your SSH public key in your AUR profile settings.
3. Clone the remote AUR repository path for your package (it will be created on first push):
   ```bash
   git clone ssh://aur@aur.archlinux.org/piopulse-bin.git
   cd piopulse-bin
   ```
4. Copy the [`packaging/aur/PKGBUILD`](file:///home/waya/Projects/PioPulse/packaging/aur/PKGBUILD) into this repository.
5. Update the `pkgver` and replace the placeholder hash with the SHA256 of `piopulse-linux-x86_64.tar.gz`.
6. Generate `.SRCINFO` (required by the AUR to index the metadata):
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```
7. Test building the package locally to ensure there are no issues:
   ```bash
   makepkg -s
   ```
8. Commit and push the package:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "release v0.1.1"
   git push origin master
   ```

### User installation:
```bash
yay -S piopulse-bin
```
