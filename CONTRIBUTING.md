# 🤝 Contributing to Deskstamp

Thank you for considering contributing to **Deskstamp**! This guide outlines how to contribute effectively and adhere to our development standards.

---

## 🎯 How to Contribute

### 🐛 Reporting Bugs

If you find a bug in Deskstamp:

1. Check the [issue tracker](https://github.com/GabrielBaiano/Deskstamp/issues) to see if it has already been reported.
2. If not, open a new issue with:
   - Clear description of the bug.
   - Steps to reproduce.
   - Your desktop environment version (`Pop!_OS 24.04 COSMIC`, `Wayland`, etc.).
   - Output logs from `deskstamp daemon`.

### ✨ Suggesting Improvements

To suggest new features:
1. Open an issue describing the proposed feature.
2. Explain the use case and how it enhances screen protection or workflow.
3. Share design ideas or mockups if applicable.

---

## 🛠️ Contribution Workflow

1. **Fork the Repository**
2. **Create a Feature Branch**:
   ```bash
   git checkout -b feat/your-feature-name
   ```
3. **Commit Changes using Conventional Commits**:
   - `feat(scope): ...`
   - `fix(scope): ...`
   - `docs(scope): ...`
4. **Ensure Verification and Tests Pass**:
   ```bash
   cargo check
   cargo test
   ```
5. **Open a Pull Request** against the `main` branch.
