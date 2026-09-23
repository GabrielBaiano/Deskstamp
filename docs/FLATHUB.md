# 📦 Publicando o Deskstamp no Flathub

Toda a infraestrutura de compilação offline (`cargo-sources.json`), manifesto Flatpak (`io.github.gabrielbaiano.Deskstamp.yml`), AppStream metainfo e ícones já estão 100% prontos e validados.

---

## 🚀 Passo a Passo para Publicação

Como você já está autenticado no `gh` (GitHub CLI), pode fazer tudo direto pelo terminal ou pelo navegador.

---

### Opção 1: Via GitHub CLI (`gh`) - Direto pelo terminal

```bash
# 1. Faça o fork do Flathub e clone na pasta /tmp ou no seu diretório de preferência
gh repo fork flathub/flathub --clone=true --remote=true /tmp/flathub
cd /tmp/flathub

# 2. Crie a branch para a nova submissão
git checkout -b new-pr/io.github.gabrielbaiano.Deskstamp

# 3. Copie o manifesto e as dependências offline
cp /home/gabrielgama/Documents/antigravity/noble-bose/io.github.gabrielbaiano.Deskstamp.yml .
cp /home/gabrielgama/Documents/antigravity/noble-bose/cargo-sources.json .

# 4. Commit e push para o seu fork
git add io.github.gabrielbaiano.Deskstamp.yml cargo-sources.json
git commit -m "Add io.github.gabrielbaiano.Deskstamp"
git push -u origin new-pr/io.github.gabrielbaiano.Deskstamp

# 5. Abra o Pull Request
gh pr create \
  --repo flathub/flathub \
  --base master \
  --title "Add io.github.gabrielbaiano.Deskstamp" \
  --body "### Summary
Deskstamp is a lightweight Wayland screen watermark overlay application designed natively for Pop!_OS COSMIC and Wayland compositors.

- **App ID**: \`io.github.gabrielbaiano.Deskstamp\`
- **Repository**: https://github.com/GabrielBaiano/Deskstamp
- **License**: GPL-3.0-or-later"
```

---

### Opção 2: Pelo Navegador + Git Manual

1. Acesse https://github.com/flathub/flathub e clique em **Fork** (canto superior direito).
2. Clone o seu fork:
   ```bash
   git clone https://github.com/GabrielBaiano/flathub.git /tmp/flathub
   cd /tmp/flathub
   git checkout -b new-pr/io.github.gabrielbaiano.Deskstamp
   ```
3. Copie os arquivos:
   ```bash
   cp /home/gabrielgama/Documents/antigravity/noble-bose/io.github.gabrielbaiano.Deskstamp.yml .
   cp /home/gabrielgama/Documents/antigravity/noble-bose/cargo-sources.json .
   ```
4. Suba as alterações:
   ```bash
   git add io.github.gabrielbaiano.Deskstamp.yml cargo-sources.json
   git commit -m "Add io.github.gabrielbaiano.Deskstamp"
   git push -u origin new-pr/io.github.gabrielbaiano.Deskstamp
   ```
5. Abra o PR pelo GitHub em https://github.com/flathub/flathub.

---

## 🤖 O que acontece depois de abrir o PR?
1. O bot do Flathub (`flathubbot`) compila automaticamente o pacote nos servidores do Flathub usando o `cargo-sources.json` offline.
2. O bot publica um link de build e um comando de teste (`flatpak install --user https://dl.flathub.org/build-repo/...`).
3. Um mantenedor do Flathub revisa e faz o merge.
4. O app entra no Flathub oficial e fica disponível na Pop!_Shop.
