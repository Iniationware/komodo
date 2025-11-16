# 🦎 Komodo Docker Deployment

Dieses Verzeichnis enthält Docker- und Docker Compose-Dateien für die Bereitstellung von Komodo Client (Agent) und Server (Periphery) in containerisierten Umgebungen.

## 📁 Dateien

- `Dockerfile.client` - Dockerfile für den Komodo Client (Agent)
- `Dockerfile.server` - Dockerfile für den Komodo Server (Periphery)
- `docker-compose.yml` - Docker Compose-Datei für beide Services
- `.env.example` - Beispiel-Umgebungsvariablen-Datei
- `README.md` - Diese Datei

## 🚀 Schnellstart

### 1. Umgebungsvariablen konfigurieren

```bash
cd docker
cp .env.example .env
# Bearbeiten Sie .env und passen Sie die Werte an Ihre Umgebung an
```

### 2. Verzeichnisse erstellen

```bash
# Erstellen Sie das Root-Verzeichnis für Periphery
mkdir -p ../komodo-root/{stacks,repos,builds}

# Erstellen Sie das Workspace-Verzeichnis für den Client (optional)
mkdir -p ../workspace
```

### 3. Container starten

```bash
# Beide Services starten
docker-compose up -d

# Nur den Server starten
docker-compose up -d komodo-server

# Nur den Client starten
docker-compose up -d komodo-client
```

### 4. Logs anzeigen

```bash
# Alle Logs
docker-compose logs -f

# Nur Server-Logs
docker-compose logs -f komodo-server

# Nur Client-Logs
docker-compose logs -f komodo-client
```

## ⚙️ Konfiguration

### Umgebungsvariablen

Beide Services können über Umgebungsvariablen konfiguriert werden. Die vollständige Liste finden Sie in:

- **Client**: [komodo.cli.toml](../config/komodo.cli.toml)
- **Server**: [periphery.config.toml](../config/periphery.config.toml)

### Konfigurationsdateien

Alternativ können Sie Konfigurationsdateien verwenden:

#### Client-Konfiguration

Erstellen Sie eine `komodo.cli.toml` Datei und mounten Sie sie:

```yaml
volumes:
  - ./config/komodo.cli.toml:/config/komodo.cli.toml
```

#### Server-Konfiguration

Erstellen Sie eine `periphery.config.toml` Datei und mounten Sie sie:

```yaml
volumes:
  - ./config/periphery.config.toml:/config/periphery.config.toml
```

## 🔐 Authentifizierung

### Server (Periphery)

Der Server benötigt Schlüssel für die Authentifizierung:

1. **Private Key**: Wird automatisch generiert, wenn nicht vorhanden
   - Standardpfad: `/config/keys/periphery.key`

2. **Core Public Key**: Erforderlich für Outbound-Modus
   - Standardpfad: `/config/keys/core.pub`
   - Muss von Komodo Core bereitgestellt werden

### Client (Agent)

Der Client benötigt API-Credentials von Komodo Core:

1. Erstellen Sie einen API-Schlüssel in der Komodo Core UI
2. Setzen Sie die Umgebungsvariablen:
   ```bash
   KOMODO_CLI_KEY=K-...
   KOMODO_CLI_SECRET=S-...
   ```

## 🌐 Netzwerk-Konfiguration

### Server-Port

Der Server lauscht standardmäßig auf Port `8120`. Sie können den Host-Port über die Umgebungsvariable ändern:

```bash
PERIPHERY_HOST_PORT=8120
```

### Netzwerk-Modi

#### Outbound-Modus (Server verbindet sich mit Core)

```bash
PERIPHERY_CORE_ADDRESS=ws://komodo-core:9120
PERIPHERY_CONNECT_AS=server-name
PERIPHERY_CORE_PUBLIC_KEYS=file:/config/keys/core.pub
```

#### Inbound-Modus (Core verbindet sich mit Server)

```bash
PERIPHERY_SERVER_ENABLED=true
PERIPHERY_PORT=8120
PERIPHERY_BIND_IP=[::]
```

## 📦 Volumes

### Client-Volumes

- `client-config` - Konfigurationsdateien
- `client-backups` - Datenbank-Backups

### Server-Volumes

- `server-config` - Konfigurationsdateien
- `server-keys` - Private/Public Keys
- `server-ssl` - SSL-Zertifikate
- `/var/run/docker.sock` - Docker Socket (für Container-Management)
- `/proc` - System-Informationen
- `PERIPHERY_ROOT_DIRECTORY` - Root-Verzeichnis für Stacks/Repos

## 🔧 Erweiterte Nutzung

### Client-Befehle ausführen

```bash
# Backup erstellen
docker-compose exec komodo-client km backup database

# Restore durchführen
docker-compose exec komodo-client km restore database

# Andere Befehle
docker-compose exec komodo-client km <command>
```

### Server-Konfiguration aktualisieren

```bash
# Konfigurationsdatei bearbeiten
docker-compose exec komodo-server vi /config/periphery.config.toml

# Server neu starten
docker-compose restart komodo-server
```

### Health Checks

Beide Services haben Health Checks konfiguriert:

```bash
# Status prüfen
docker-compose ps

# Health Check Details
docker inspect komodo-server | jq '.[0].State.Health'
```

## 🐛 Troubleshooting

### Server startet nicht

1. Prüfen Sie die Logs:
   ```bash
   docker-compose logs komodo-server
   ```

2. Stellen Sie sicher, dass der Docker Socket gemountet ist:
   ```yaml
   volumes:
     - /var/run/docker.sock:/var/run/docker.sock:ro
   ```

3. Prüfen Sie die Berechtigungen für `/proc`:
   ```yaml
   volumes:
     - /proc:/proc:ro
   ```

### Client kann sich nicht verbinden

1. Prüfen Sie die Core-Adresse:
   ```bash
   echo $KOMODO_CLI_HOST
   ```

2. Stellen Sie sicher, dass die API-Credentials gesetzt sind:
   ```bash
   echo $KOMODO_CLI_KEY
   echo $KOMODO_CLI_SECRET
   ```

3. Testen Sie die Verbindung:
   ```bash
   docker-compose exec komodo-client km --version
   ```

### Schlüssel-Probleme

Wenn der Server sich nicht authentifizieren kann:

1. Prüfen Sie, ob die Schlüssel existieren:
   ```bash
   docker-compose exec komodo-server ls -la /config/keys/
   ```

2. Stellen Sie sicher, dass die Core Public Key korrekt ist:
   ```bash
   docker-compose exec komodo-server cat /config/keys/core.pub
   ```

## 📚 Weitere Informationen

- [Komodo Dokumentation](https://komo.do/docs)
- [Client Konfiguration](../config/komodo.cli.toml)
- [Server Konfiguration](../config/periphery.config.toml)
- [GitHub Repository](https://github.com/moghtech/komodo)

## 📝 Lizenz

GPL-3.0

