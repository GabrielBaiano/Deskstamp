# 🤝 Contributing to Deskstamp

Thank you for contributing to **Deskstamp**! This guide outlines our development standards, issue reporting process, and pull request workflow.

---

## 🎯 How to Contribute

### 🐛 Reporting Bugs

Before creating an issue, search the [existing issues](https://github.com/GabrielBaiano/Deskstamp/issues) to avoid duplicates.

When reporting a bug, use the **Bug Report** template:
1. Provide a clear summary and minimal steps to reproduce.
2. Include your environment details:
   - Distro / OS (`Pop!_OS 24.04 LTS`, `Arch Linux`, `Fedora`, etc.)
   - Desktop Environment (`COSMIC Desktop`, `GNOME`, `Sway`, `Hyprland`)
   - Wayland compositor (`cosmic-comp`, `wlroots`, `mutter`)
   - Deskstamp version (`deskstamp --version`)
3. Attach terminal logs from running `deskstamp daemon` or `deskstamp test`.

### 💡 Requesting Features

To propose an idea or improvement, open an issue using the **Feature Request** template:
1. Explain the problem or use case.
2. Outline the proposed solution and how it should behave.
3. Include any UI/UX mockups, CLI syntax ideas, or reference examples if applicable.

---

## 🛠️ Development & Pull Request Workflow

### 1. Branching Strategy
- Fork the repository and create your branch from `main`:
  ```bash
  git checkout -b feat/your-feature-name
  # or
  git checkout -b fix/issue-description
  ```

### 2. Conventional Commits
All commits should follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:
- `feat(scope): add new watermark pattern`
- `fix(renderer): resolve rotation clipping at negative angles`
- `docs(readme): clarify pop-os cosmic installation steps`
- `refactor(gui): streamline settings layout`
- `test(config): add unit tests for custom icon parser`
- `chore: update dependencies`

Common scopes: `gui`, `renderer`, `config`, `daemon`, `cli`, `obs`.

### 3. Verification & Code Quality
Before opening a Pull Request, verify your changes locally:
```bash
cargo fmt --check
cargo check
cargo test
```
Also verify the overlay behavior manually under Wayland:
```bash
cargo run -- test
```

### 4. Submitting a Pull Request
1. Fill out the provided **Pull Request Template** (`.github/PULL_REQUEST_TEMPLATE.md`).
2. Link the relevant issue (e.g., `Fixes #12`).
3. Ensure all CI checks pass.
4. Keep PRs focused on a single concern for faster reviews.

