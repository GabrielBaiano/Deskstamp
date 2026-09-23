# 📦 Publicando o Deskstamp no Flathub

Toda a infraestrutura de compilação offline (`cargo-sources.json`), manifesto Flatpak (`io.github.gabrielbaiano.Deskstamp.yml`), AppStream metainfo e ícones já estão 100% prontos e validados.

---

## 🚀 Passo a Passo Rápido

### 1. Faça o Fork do repositório oficial do Flathub
Acesse https://github.com/flathub/flathub e clique em **Fork** (no canto superior direito).

### 2. Clone o seu fork na sua máquina
```bash
git clone https://github.com/GabrielBaiano/flathub.git
cd flathub
```

### 3. Crie uma branch para o Deskstamp
```bash
git checkout -b new-pr/io.github.gabrielbaiano.Deskstamp
```

### 4. Copie os arquivos de manifesto e compilação offline do Deskstamp para o fork
```bash
# Na raiz do repositório 'flathub':
cp /home/gabrielgama/Documents/antigravity/noble-bose/io.github.gabrielbaiano.Deskstamp.yml .
cp /home/gabrielgama/Documents/antigravity/noble-bose/cargo-sources.json .
```

### 5. Commit e Push no seu fork
```bash
git add io.github.gabrielbaiano.Deskstamp.yml cargo-sources.json
git commit -m "Add io.github.gabrielbaiano.Deskstamp"
git push -u origin new-pr/io.github.gabrielbaiano.Deskstamp
```

### 6. Abra o Pull Request
1. Acesse o seu fork no GitHub: https://github.com/GabrielBaiano/flathub
2. O GitHub mostrará um botão verde: **"Compare & pull request"**. Clique nele.
3. No título do PR, coloque:
   ```text
   Add io.github.gabrielbaiano.Deskstamp
   ```
4. No corpo do PR, você pode usar uma descrição simples:
   ```markdown
   ### Summary
   Deskstamp is a lightweight Wayland screen watermark overlay application designed natively for Pop!_OS COSMIC and Wayland compositors.

   - **App ID**: `io.github.gabrielbaiano.Deskstamp`
   - **Repository**: https://github.com/GabrielBaiano/Deskstamp
   - **License**: GPL-3.0-or-later
   ```
5. Clique em **Create pull request**.

---

## 🤖 O que acontece depois de abrir o PR?
1. O bot do Flathub (`flathubbot`) compila automaticamente o pacote nos servidores do Flathub usando o `cargo-sources.json` offline.
2. O bot comentará no PR com um comando para você testar a instalação do Flatpak gerado se quiser (`flatpak install --user https://dl.flathub.org/build-repo/...`).
3. Um revisor do Flathub aprova o PR.
4. O app é publicado oficialmente no Flathub e fica disponível na Pop!_Shop e via `flatpak install flathub io.github.gabrielbaiano.Deskstamp`.
