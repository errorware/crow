### name: Spanish
### native: Español
### author: Crow (first translation; corrections welcome)
### crow: 0.8.2

## appearance
appearance-choose-a-picture-for-the = Elige una imagen para la página Flota

## app-archive
app-archive-stored-data-purged = datos guardados eliminados
app-archive-data-purged-in-days = los datos se eliminan en { $days } días
app-archive-data-purged-in-1-day = los datos se eliminan en 1 día
app-archive-data-purged-today = los datos se eliminan hoy
app-archive-purge-due = eliminación pendiente
app-archive-purging-is-off-data-kept = la eliminación está desactivada · datos conservados
app-archive-never-purging-is-switched-off = nunca — la eliminación está desactivada
app-archive-is-the-bastion-for-move = { $name } es el bastión de { $users }: muévelos primero y luego archívalo
app-archive-could-not-archive = No se pudo archivar { $name }: { $e }
app-archive-could-not-archive-the-vault = No se pudo archivar: la bóveda no está disponible
app-archive-archived-its-data-is-purged = { $name } archivado · sus datos se eliminan { $purge_due_text }
app-archive-could-not-restore = No se pudo restaurar { $name }: { $e }
app-archive-restored-to-the-fleet = { $name } vuelve a estar en la flota
app-archive-could-not-purge-s-data = No se pudieron eliminar los datos de { $name }: { $e }

## app-bastions
app-bastions-is-still-the-bastion-for = { $name } sigue siendo el bastión de { $users }: muévelos primero
app-bastions-is-a-bastion-servers-can = { $name } es un bastión: se pueden añadir servidores a través de él
app-bastions-is-no-longer-a-bastion = { $name } ya no es un bastión

## app-certificates
app-certificates-isnt-managed-by-certbot-or = { $cert_path } no lo gestiona certbot ni acme.sh en { $name }: renuévalo donde se emitió.
app-certificates-certbot-dry-run-renew-reload = certbot: prueba en seco, renovar { $name }, recargar{ $inv }{ $inv2 }, confirmar
app-certificates-acme-sh-renew-reload-confirm = acme.sh: renovar { $domain }, recargar, confirmar
app-certificates-renew = RENOVAR { $label }

## app-checks
app-checks-http-a-word-the-page = HTTP: una palabra que la página debe contener · DNS: la dirección a la que debe resolver
app-checks-name-optional = nombre (opcional)
app-checks-type-what-to-check-first = Escribe primero qué comprobar.
app-checks-an-http-check-needs-a = Una comprobación HTTP necesita una URL que empiece por http:// o https://.
app-checks-a-tcp-check-is-host = Una comprobación TCP es host:puerto, como db.lan:5432.
app-checks-added-checked-every-s-while = Añadida «{ $name }»: se comprueba cada { $every_secs } s mientras Crow está abierto.

## app-clankers
app-clankers-set-leave-blank-to-keep = configurada · déjala en blanco para conservarla, o pega una clave nueva
app-clankers-e-g-sk-proj = p. ej. sk-proj-...
app-clankers-e-g-gpt-4o-mini = p. ej. gpt-4o-mini
app-clankers-model-cannot-be-empty = El modelo no puede estar vacío
app-clankers-the-vault-is-busy-try = la bóveda está ocupada; inténtalo de nuevo
app-clankers-unknown-provider = proveedor desconocido
app-clankers-no-encryption-key = no hay clave de cifrado
app-clankers-not-saved = No guardado: { $e }
app-clankers-is-the-primary-clanker = { $name } es el clanker principal
app-clankers-is-the-backup-clanker = { $display_name } es el clanker de respaldo
app-clankers-no-backup-clanker = Sin clanker de respaldo
app-clankers-provider-call-stats-reset = Estadísticas de llamadas del proveedor reiniciadas
app-clankers-no-ai-provider-has-an = Ningún proveedor de IA tiene clave de API todavía. Añade una arriba con ⚙ Editar clave.
app-clankers-via-the-backup-the-primary =
    mediante el respaldo, { $provider_label } (el principal falló: { $why })
    {""}
    { $text }
app-clankers-via =
    mediante { $provider_label }
    {""}
    { $text }

## app-configs
app-configs-the-fleet = la flota
app-configs-group = grupo { $scope }
app-configs-type-a-key-or-pick = escribe una clave o elige una de la lista
app-configs-a-directive-needs-its-arguments = una directiva necesita sus argumentos
app-configs-could-not-be-parsed = no se pudo interpretar { $selected }: { $e }
app-configs-no-config-file-selected = Ningún archivo de configuración seleccionado.
app-configs-isnt-one-of-the-files = { $path } no es uno de los archivos que Crow gestiona en este servidor
app-configs-this-servers-configs-are-still = las configuraciones de este servidor aún se están cargando; no se escribió nada
app-configs-is-not-loaded = { $file } no está cargado
app-configs-couldnt-re-read-it = { $file }: no se pudo volver a leer: { $e }
app-configs-not-saved = { $file }: no guardado: { $e }
app-configs-filter-configs = Filtrar configuraciones...
app-configs-reverted-firewall-rules-to-baseline = Reglas del cortafuegos devueltas a la línea base
app-configs-the-vault-is-busy = la bóveda está ocupada
app-configs-no-snapshot-nothing-applied = Sin instantánea, no se aplicó nada: { $e }
app-configs-the-snapshot-failed-so-nothing = La instantánea falló, así que no se aplicó nada: { $e }
app-configs-type-exactly-to-stage-this = Escribe { $risk_confirm_keyword } exactamente para preparar este cambio.
app-configs-audit-commit-created = Registro de auditoría creado: { $description }
app-configs-rolled-back-firewall-to-revision = Cortafuegos devuelto a la revisión v{ $version }
app-configs-this-machine = este equipo
app-configs-was-written-but-its-history = { $file } se escribió, pero su historial no se guardó: { $e }
app-configs-baseline-not-pinned = línea base no fijada: { $e }
app-configs-baseline-not-unpinned = línea base no liberada: { $e }
app-configs-couldnt-be-read-or-recorded = no se pudo leer ni registrar: { $e }

## app-containers
app-containers-done = Hecho: { $summary }
app-containers-failed = { $summary } falló: { $e }
app-containers-no-ai-provider-has-a = Ningún proveedor de IA tiene clave todavía. Añade una en Ajustes → Clankers (IA).
app-containers-every-network = todas las redes
app-containers-this-machine-only = solo este equipo

## app-danger
app-danger-the-vault-is-busy = la bóveda está ocupada
app-danger-asking-to = pidiendo a { $label } que { $verb }…
app-danger-type-exactly-to-confirm = Escribe { $keyword } exactamente para confirmar.
app-danger-no-active-server = Ningún servidor activo
app-danger-stopped-lab-container-s-failed = { $ok } contenedor(es) de laboratorio detenido(s), { $failed } fallaron
app-danger-unknown-action = Acción desconocida

## app-drift
app-drift-a-file-sshd-config-or = un archivo (sshd_config) o un ajuste (PermitRootLogin) en cualquier servidor…
app-drift-couldnt-read-on-a-server = no se pudo leer { $failed } en un servidor: { $failed2 }
app-drift-the-baselines-content-wasnt-kept = no se guardó el contenido de la línea base (solo su hash), así que no hay nada que enviar
app-drift-not-reachable-right-now = no accesible ahora mismo
app-drift-put-back-to-the-baseline = devolver { $path } a la línea base de { $e }
app-drift-bring-back = RESTAURAR { $path } · { $n } { $n ->
        [one] SERVIDOR
       *[other] SERVIDORES
    }
app-drift-the-vault-is-busy = la bóveda está ocupada
app-drift-crow-has-no-copy-of = Crow no tiene ninguna copia de ese servidor
app-drift-only-the-hash-of-that = solo se guardó el hash de esa copia, así que no se pudo enviar a ningún sitio: abre la bóveda y léela de nuevo
app-drift-s-is-now-the-baseline = { $path } de { $name } es ahora la línea base de { $app }. A partir de ahora los demás servidores se comparan con ella.
app-drift-not-accepted = No aceptado: { $e }

## app-dr
app-dr-plan-name-e-g-orders = nombre del plan, p. ej. base de datos de pedidos
app-dr-rto-minutes = RTO, minutos
app-dr-rpo-minutes = RPO, minutos
app-dr-drill-notes-e-g-restored = notas del simulacro, p. ej. restaurado en una VM de Multipass desde el volcado de anoche

app-dr-what-matters-one-per-line =
    lo que importa, uno por línea:
    /var/lib/postgresql
    /etc/nginx
    base de datos de pedidos
app-dr-backups-one-per-line-path =
    copias de seguridad, una por línea (ruta o patrón | temporizador):
    /var/backups/db/*.sql.gz | pg-dump.timer
app-dr-rebuild-1-launch-a-fresh =
    ## Reconstrucción
    1. Lanza un servidor nuevo (Añadir servidor)
    2. Restaura: zcat /var/backups/db/latest.sql.gz | psql orders
    3. Envía las líneas base (Desviación → restaurar)
    4. Comprueba el sitio (Comprobaciones)
app-dr-give-the-plan-a-name = Ponle un nombre al plan y elige qué cubre.
app-dr-saved = Guardado.
app-dr-save-the-plan-first = Guarda el plan primero.
app-dr-not-reachable-right-now-nothing = no accesible ahora mismo: no se pudo comprobar nada
app-dr-drill-started-follow-the-runbook = Simulacro iniciado: sigue el procedimiento (restaura en una VM de Multipass o en un servidor de pruebas) y luego TERMINAR SIMULACRO.
app-dr-drill-recorded-min-the-rto = Simulacro registrado: { $minutes } min, { $met ->
        [yes] dentro del
       *[no] por encima del
    } RTO de { $rto_minutes } min.
app-dr-group = grupo { $g }
app-dr-plan-written-to-keep-a = Plan guardado en { $path }: conserva una copia donde puedas leerla cuando Crow no funcione.
app-dr-couldnt-write = No se pudo escribir { $path }: { $e }

## app-files
app-files-chmod = chmod { $after } { $path }: { $e }
app-files-that-name-has-control-characters = Ese nombre tiene caracteres de control; ábrelo tú desde la terminal.

## app-firewall
app-firewall-8080-or-a-range-8000 = 8080, o un rango 8000:8100
app-firewall-anywhere-or-an-address-cidr = cualquier origen, o una dirección / CIDR
app-firewall-why-this-rule-exists = por qué existe esta regla
app-firewall-reading-s-firewall = Leyendo el cortafuegos de { $name }…
app-firewall-ran = Ejecutado: { $summary }
app-firewall-failed = Falló: { $summary } — { $e }
app-firewall-type-exactly-to-apply-this = Escribe { $risk_confirm_keyword } exactamente para aplicar este cambio.
app-firewall-not-applied-no-firewall-crow = No aplicado: no se encontró en este host ningún cortafuegos que Crow pueda cambiar.
app-firewall-not-applied-crow-reads-but = No aplicado: Crow lee { $label } pero nunca lo escribe.
app-firewall-not-applied = No aplicado: { $e }

## app-fleet-run
app-fleet-run-the-baselines-content-wasnt-kept = { $path }: no se guardó el contenido de la línea base (solo el hash), así que no hay nada que enviar
app-fleet-run-push-baseline = enviar la línea base { $path }
app-fleet-run-rolling-reboot = REINICIO ESCALONADO · { $n } { $n ->
        [one] SERVIDOR
       *[other] SERVIDORES
    }
app-fleet-run-the-machine-crow-runs-on = el equipo donde corre Crow: reinícialo tú
app-fleet-run-not-reachable-right-now-so = no accesible ahora mismo, así que Crow no podría verlo volver
app-fleet-run-reboot-via = reiniciar mediante { $label }
app-fleet-run-restart-the-lab-container = reiniciar el contenedor de laboratorio
app-fleet-run-the-machine-crow-runs-on-2 = el equipo donde corre Crow
app-fleet-run-a-lab-container-reached-without = un contenedor de laboratorio, accesible sin SSH
app-fleet-run-not-reachable-right-now-the = no accesible ahora mismo: las claves nuevas se obtienen por la conexión actual
app-fleet-run-new-host-keys-re-pin = claves de host nuevas, volver a fijarlas, verificar un inicio de sesión nuevo
app-fleet-run-not-pinned = sin fijar
app-fleet-run-other-machines = otros equipos
app-fleet-run-anyone-else-who-sshes-to = cualquiera que se conecte por SSH a estos servidores verá una clave de host distinta y tendrá que volver a verificarla
app-fleet-run-rotate-host-keys = ROTAR CLAVES DE HOST · { $n } { $n ->
        [one] SERVIDOR
       *[other] SERVIDORES
    }
app-fleet-run-the-machine-crow-runs-on-3 = el equipo donde corre Crow (tus propias sesiones)
app-fleet-run-not-reachable-right-now = no accesible ahora mismo
app-fleet-run-end-every-ssh-login-but = cerrar todas las sesiones SSH excepto la de Crow
app-fleet-run-no-key-is-removed-anyone = no se elimina ninguna clave: quien tenga una clave que aún funcione puede volver a entrar (para eso, rota o elimina claves)
app-fleet-run-revoke-sessions = REVOCAR SESIONES · { $n } { $n ->
        [one] SERVIDOR
       *[other] SERVIDORES
    }
app-fleet-run-crow-has-only-the-public = Crow solo tiene la mitad pública de esta clave, así que no puede iniciar sesión con ella
app-fleet-run-no-server-logs-in-with = Ningún servidor inicia sesión con { $name }; no hay nada que rotar.
app-fleet-run-couldnt-generate-the-new-key = No se pudo generar la clave nueva: { $e }
app-fleet-run-new-key-written-to-if = clave nueva guardada en { $dir }; si cancelas, se queda ahí sin usar
app-fleet-run-install-new-key-verify-switch = instalar la clave nueva, verificar, cambiar, eliminar la clave antigua
app-fleet-run-install-key-verify-switch = instalar la clave, verificar, cambiar
app-fleet-run-switched-old-key-removed = cambiada; clave antigua eliminada
app-fleet-run-type-exactly-to-start = Escribe { $keyword } exactamente para empezar.
app-fleet-run-approved-by = Aprobado por { $by }.
app-fleet-run-crow = crow: { $title } — { $summary }
app-fleet-run-pinned-was = fijadas { $r } (antes { $before })

## app-groups
app-groups-group-name-e-g-workers = nombre del grupo (p. ej. workers, edge, data)

## app-hardening
app-hardening-why-this-risk-is-acceptable = por qué este riesgo es aceptable (necesario para aceptarlo)
app-hardening-write-why-the-risk-is = Escribe primero por qué el riesgo es aceptable: el motivo va al informe y al registro de auditoría.
app-hardening-accepted-its-no-longer-flagged = Aceptado: ya no se marca en el mapa ni aquí, y el informe lo incluye con tu motivo.
app-hardening-accepted-as-a-risk = aceptado como riesgo
app-hardening-not-an-ssh-server-crow = no es un servidor SSH (Crow comprueba cada corrección con un inicio de sesión SSH nuevo)
app-hardening-not-reachable-right-now = no accesible ahora mismo
app-hardening-neither-ufw-nor-firewalld-is = no está instalado ni ufw ni firewalld: instala uno (apt install ufw, dnf install firewalld) y luego corrígelo
app-hardening-crow-logs-in-as-root = Crow entra como root con una clave: prohibit-password ya es el ajuste más seguro que puede mantener
app-hardening-report-written-to = Informe guardado en { $path }
app-hardening-couldnt-write-the-report-to = No se pudo guardar el informe en { $path }: { $e }

## app-history-view
app-history-view-disk = DISCO { $mount }

## app-incidents
app-incidents-on = { $detail } en { $name }
app-incidents-days = { $secs } días
app-incidents-hours = { $secs } horas
app-incidents-no-ai-provider-has-a = Ningún proveedor de IA tiene clave todavía. Añade una en Ajustes → Clankers (IA).
app-incidents-timeline-written-to = Cronología guardada en { $path }
app-incidents-couldnt-write = No se pudo escribir { $path }: { $e }
app-incidents-the-last = { $name }, últimos { $secs }

## app-journal
app-journal-no-ai-provider-has-an = Ningún proveedor de IA tiene clave de API. Añade una en Ajustes → Clankers.
app-journal-there-are-no-log-lines = No hay líneas de registro en pantalla que explicar.
app-journal-and-worse = { $label } y peores
app-journal-answered-by-the-backup-the =
    Respondido por el respaldo, { $provider_label }: el principal falló ({ $why }).
    {""}
    { $text }
app-journal-nothing-to-export = Nada que exportar
app-journal-copied = { $lines ->
        [one] Copiada { $lines } línea
       *[other] Copiadas { $lines } líneas
    }

## app-keys
app-keys-enrolled-into-group = «{ $name }» añadida al grupo «{ $target_group }»
app-keys-removed-key-from-memory-file = Clave quitada de la memoria (el archivo se conserva)

app-keys-key-name-cannot-be-empty = El nombre de la clave no puede estar vacío
app-keys-group-name-cannot-be-empty = El nombre del grupo no puede estar vacío
app-keys-path-cannot-be-empty = La ruta no puede estar vacía
app-keys-directory-does-not-exist = La carpeta no existe: { $expanded }
app-keys-e-g-id-ed25519-bastion = p. ej. id_ed25519_bastion
app-keys-e-g-nelson-crow = p. ej. nelson@crow
app-keys-e-g-staging-fleet-edge = p. ej. Flota de pruebas, Bastiones de borde
app-keys-e-g-work-keys-or = p. ej. ~/work-keys o /etc/ssh
app-keys-enter-key-name = Nombre de la clave…

## app-lab
app-lab-crows-key = Clave de Crow: { $e }
app-lab-multipass = Multipass: { $e }
app-lab-couldnt-save-in-the-vault = no se pudo guardar { $name } en la bóveda: { $e }
app-lab-is-in-the-fleet = { $name } está en la flota ({ $host })
app-lab-launching = Lanzando { $name }
app-lab-importing = Importando { $name }
app-lab-deleted = { $vm } eliminada
app-lab-done = { $vm }: { $label } hecho

## app-logsearch
app-logsearch-text-or-regex-e-g = texto o regex, p. ej. Out of memory|segfault
app-logsearch-unit-optional-e-g-nginx = unidad (opcional), p. ej. nginx.service
app-logsearch-a-name-to-save-it = un nombre para guardarla
app-logsearch-no-reachable-server-in = Ningún servidor accesible en { $label }.
app-logsearch-couldnt-read-its-journal-no = no se pudo leer su journal (sin journalctl, o inaccesible)
app-logsearch-name-the-search-to-save = Ponle nombre a la búsqueda para guardarla.
app-logsearch-a-saved-search-needs-text = Una búsqueda guardada necesita texto, una unidad o una prioridad: una que coincida con todo avisaría siempre.
app-logsearch-saved-switch-alert-on-to = «{ $name }» guardada. Activa ALERTA para enterarte cuando coincida.
app-logsearch-no-ai-provider-has-a = Ningún proveedor de IA tiene clave todavía. Añade una en Ajustes → Clankers (IA).

## app
app-no-server-is-enrolled-yet = Todavía no hay ningún servidor añadido.
app-detecting = Detectando...

## app-notify
app-notify-blank-if-the-server-needs = en blanco si el servidor no necesita inicio de sesión
app-notify-set-leave-blank-to-keep = configurada · déjala en blanco para conservarla
app-notify-the-smtp-password-or-app = la contraseña SMTP o de aplicación
app-notify-you-example-com-oncall-example = tu@ejemplo.com, guardia@ejemplo.com
app-notify-e-g-22-07-or = p. ej. 22-07, o en blanco
app-notify-the-form-isnt-open = el formulario no está abierto
app-notify-the-port-is-a-number = el puerto es un número, como 465 o 587
app-notify-the-digest-interval-is-a = el intervalo del resumen es un número de minutos
app-notify-theres-a-username-but-no = hay un usuario pero no una contraseña
app-notify-the-vault-is-busy-try = la bóveda está ocupada; inténtalo de nuevo
app-notify-the-vault-is-locked = la bóveda está bloqueada
app-notify-saved-alerts-are-mailed-while = Guardado. Las alertas se envían por correo mientras Crow está en marcha.
app-notify-not-saved = No guardado: { $e }
app-notify-the-saved-password-cant-be = la contraseña guardada no se puede leer con la bóveda bloqueada
app-notify-sent-to-check-the-inbox = Enviado a { $to }. Revisa la bandeja de entrada (y el spam).
app-notify-nothing-was-sent = no se envió nada
app-notify-the-smtp-password-cant-be = no se puede leer la contraseña SMTP (¿está bloqueada la bóveda?)

## app-onboard
app-onboard-e-g-10-0-4 = p. ej. 10.0.4.32 o prod-db.internal
app-onboard-e-g-prod-db-01 = p. ej. prod-db-01
app-onboard-e-g-postgres-primary = p. ej. postgres, primary
app-onboard-test-node = nodo de pruebas · { $distro }
app-onboard-the-servers-address-is-missing = Falta la dirección del servidor.
app-onboard-that-isnt-an-address-a = Eso no es una dirección: un nombre de host o una IP, sin espacios.
app-onboard-the-user-to-log-in = Falta el usuario con el que iniciar sesión.
app-onboard-pick-the-key-crow-logs = Elige la clave con la que entra Crow, o usa una contraseña una sola vez.
app-onboard-type-the-servers-password-crow = Escribe la contraseña del servidor: Crow la usa una vez para instalar su clave.
app-onboard-trust-the-servers-host-key = Primero confía en la clave de host del servidor (o vuelve a comprobarla).
app-onboard-give-it-a-name = Ponle un nombre.
app-onboard-is-already-the-name-of = { $name } ya es el nombre de otro servidor.
app-onboard-no-home-directory = no hay carpeta personal
app-onboard-no-free-crow-ed25519-file = no queda ningún nombre de archivo crow_ed25519 libre en ~/.ssh
app-onboard-saving-crows-key-in-the = guardando la clave de Crow en la bóveda: { $e }
app-onboard-enter-the-servers-password-on = introduce la contraseña del servidor en el paso 2 (Credenciales)
app-onboard-the-bastion = el bastión
app-onboard-cant-go-through-the-bastion = no se puede pasar por el bastión: { $e }
app-onboard-the-host-key-changed-remove = La clave de host cambió. Si el cambio es esperado, quita tú la entrada antigua de ~/.ssh/known_hosts.
app-onboard-no-host-key-was-fetched = No se obtuvo ninguna clave de host del servidor; vuelve a sondearlo.
app-onboard-couldnt-write-ssh-known-hosts = No se pudo escribir ~/.ssh/known_hosts: { $e }
app-onboard-server-enrolled-into-fleet = Servidor «{ $name }» añadido a la flota
app-onboard-password-login = inicio de sesión con contraseña: { $why }
app-onboard-logged-in-with-the-password = sesión iniciada una vez con la contraseña; clave de Crow instalada ({ $fingerprint })
app-onboard-key-only-from-now-on = solo con clave a partir de ahora
app-onboard-password-login-2 = inicio de sesión con contraseña: { $e }

## app-overview
app-overview-search-services = buscar servicios…  ( / )
app-overview-search-command-user-pid = buscar comando, usuario, pid…  ( / )
app-overview-search-name-group-shell = buscar nombre, grupo, shell…  ( / )
app-overview-search-port-address-comment = buscar puerto, dirección, comentario…  ( / )
app-overview-couldnt-read-updates = no se pudieron leer las actualizaciones: { $e }
app-overview-failed = { $failed } CON FALLOS
app-overview-on = ACTIVO
app-overview-off = INACTIVO
app-overview-established-listening = { $established } establecidas, { $listening } escuchando
app-overview-no-ai-provider-has-a = Ningún proveedor de IA tiene clave todavía. Añade una en Ajustes → Clankers (IA).

## app-palette
app-palette-page = PÁGINA
app-palette-this-server = este servidor
app-palette-danger-zone = Zona de peligro
app-palette-offline-unreachable-down = desconectado inaccesible caído offline unreachable down
app-palette-server = SERVIDOR
app-palette-terminal = TERMINAL
app-palette-terminal-on = Terminal en { $name }
app-palette-place = LUGAR
app-palette-fleet = Flota
app-palette-fleet-map = Mapa de la flota
app-palette-fleet-setup-policies = Grupos y políticas
app-palette-patching = Parches
app-palette-rollouts = Despliegues
app-palette-drift-config-search = Desviación y búsqueda de configuración
app-palette-audit-log = Registro de auditoría
app-palette-settings = Ajustes
app-palette-ssh-keys = Claves SSH
app-palette-fleet-config = CONFIG. DE FLOTA
app-palette-action = ACCIÓN
app-palette-add-a-server = Añadir un servidor
app-palette-check-posture-now = Comprobar la postura ahora
app-palette-reconnect-to-this-server = Reconectar con este servidor
app-palette-lock-the-vault = Bloquear la bóveda
app-palette-take-a-snapshot = Hacer una instantánea
app-palette-security-stance-2fa = Postura de seguridad (2FA)
app-palette-about-crow = Acerca de Crow
app-palette-key = CLAVE
app-palette-config = CONFIG.
app-palette-service = SERVICIO
app-palette-process = PROCESO
app-palette-pid = pid { $pid } · { $user }
app-palette-search-servers-pages-config-files = Busca servidores, páginas, archivos de configuración, acciones, claves…

## app-password-login

app-password-login-crow-doesnt-connect-to-this = Crow todavía no se conecta a este servidor con su propia clave. Configura primero el acceso con clave (Añadir servidor → instalar la clave de Crow) y luego desactiva las contraseñas.
app-password-login-password-login-is-already-off = ✓ El inicio de sesión con contraseña ya está desactivado en este servidor.
app-password-login-this-sshd-config-doesnt-include = Este sshd_config no incluye sshd_config.d, así que el archivo adicional de Crow no tendría efecto. Pon PasswordAuthentication en no en la hoja de sshd_config y luego recarga sshd.
app-password-login-couldnt-read-sshds-settings = No se pudieron leer los ajustes de sshd: { $e }

## app-patching
app-patching-every-server = todos los servidores
app-patching-group = grupo { $g }
app-patching-any-time-e-g-sun = a cualquier hora · p. ej. Sun 02-05
app-patching-the-machine-crow-runs-on = el equipo donde corre Crow: parchéalo tú
app-patching-not-reachable-right-now = no accesible ahora mismo
app-patching-its-updates-havent-been-read = aún no se han leído sus actualizaciones: primero ACTUALIZAR
app-patching-couldnt-read-its-updates = no se pudieron leer sus actualizaciones: { $e }
app-patching-no-pending = no hay { $label } pendientes
app-patching-cant-be-rebooted-from-here = no se puede reiniciar desde aquí ({ $why }), así que queda fuera de una ejecución con reinicio
app-patching-then-reboot-if-needed = {" "}· luego reiniciar si hace falta
app-patching-rebooted = { $done }; reiniciado: { $m }

## app-people
app-people-failed-exit = `{ $cloned }` falló (salida { $status }): { $stderr }
app-people-a-keys-fingerprint-sha256-or = la huella de una clave (SHA256:…) o su comentario
app-people-username-e-g-alice = usuario, p. ej. alicia
app-people-their-public-key-ssh-ed25519 = su clave pública: ssh-ed25519 AAAA… alicia@portatil
app-people-revoke-keys-lock-and-expire = revocar claves, bloquear y caducar
app-people-revoke-keys-delete-the-account = revocar claves, eliminar la cuenta (se conserva la carpeta personal)
app-people-text = { $user }: { $verb } ({ $count } { $count ->
        [one] paso
       *[other] pasos
    })
app-people-remove-the-key-from = quitar la clave de { $count } { $count ->
        [one] cuenta
       *[other] cuentas
    }
app-people-fill-in-the-username-and = Rellena primero el usuario y su clave pública.
app-people-no-server-in-has-been = Todavía no se ha leído ningún servidor de { $label }: primero LEER DE NUEVO.
app-people-create-add-their-key = crear { $user }{ $sudo ->
        [yes] {" "}con sudo
       *[no] {""}
    }, añadir su clave
app-people-add-the-key-to = añadir la clave a { $user }

## app-plugins
app-plugins-local-vms-containers = VMs y contenedores locales
app-plugins-multipass-vms = VMs de Multipass
app-plugins-local-lab-containers = Contenedores de laboratorio locales
app-plugins-is-on = está activado
app-plugins-is-off = está desactivado
app-plugins-mails = envía a { $s }
app-plugins-not-set-up-yet = sin configurar todavía
app-user-plugins-couldnt-create = No se pudo crear { $path }: { $e }
app-user-plugins-changed-while-reviewing = { $name } cambió mientras lo revisabas. Revísalo de nuevo.
app-user-plugins-accepted = { $name } aceptado: sus archivos quedan fijados.
app-user-plugins-refused = { $name } rechazado.
app-user-plugins-revoked = { $name } ya no está aceptado.
app-user-plugins-state-new = NUEVO
app-user-plugins-state-accepted = ACEPTADO
app-user-plugins-state-changed = CAMBIADO · REVISAR DE NUEVO
app-user-plugins-state-refused = RECHAZADO
app-user-plugins-state-invalid = NO SE PUEDE CARGAR
app-user-plugins-not-accepted-now = Los archivos de este plugin cambiaron desde que lo aceptaste, así que está desactivado. Revísalo en Ajustes → Plugins.
app-user-plugins-later-stage = Las acciones que cambian un servidor, o que se ejecutan en esta máquina, llegarán en una versión posterior.
app-user-plugins-needs-settings = Faltan ajustes: { $names }
app-user-plugins-needs-input = Falta un valor para { $name }
app-user-plugins-dry-run-first = Haz primero el ensayo, con estos valores.
app-user-plugins-dry-run-still-going = El ensayo sigue en marcha.
app-user-plugins-dry-run-failed = El ensayo falló: mira su salida antes de ejecutar esto.
app-user-plugins-type-keyword = Escribe { $keyword } para ejecutarlo.
app-user-plugins-stopped = detenido
app-user-plugins-needs-install = Instala primero este plugin (INSTALAR): su instalación no se ha ejecutado con estos archivos.
app-user-plugins-missing-here = Necesita en esta máquina: { $names }
app-user-plugin-settings-secret-saved = guardado · escribe para cambiarlo
app-user-plugin-settings-needed = obligatorio
app-user-plugin-settings-vault-locked = Desbloquea la bóveda para guardar secretos.
app-user-plugin-settings-saved = { $name }: ajustes guardados.
app-user-plugin-fleet-done = hecho
app-user-plugin-fleet-failed = falló
app-user-plugin-fleet-no-ssh = un contenedor de laboratorio: las acciones en esta máquina necesitan SSH
app-user-plugin-fleet-no-servers = No hay servidor donde ejecutarlo: cambia a dónde va.
app-user-plugin-fleet-no-result = el plugin no escribió ningún resultado para este servidor
app-user-plugin-fleet-title = { $label } · { $plugin }

## app-providers
app-providers-set-type-to-replace = configurado · escribe para sustituirlo
app-providers-not-set = sin configurar
app-providers-default = por defecto: { $d }
app-providers-the-vault-is-busy-try = la bóveda está ocupada; inténtalo de nuevo
app-providers-the-vault-is-locked-or = la bóveda está bloqueada o no tiene contraseña, así que no se pueden leer los secretos del proveedor
app-providers-no-provider-accounts-yet-set = Todavía no hay cuentas de proveedor. Configura una en Ajustes → Proveedores.
app-providers-no-encryption-key = no hay clave de cifrado
app-providers-this-provider-cant-list-instances = este proveedor no puede listar instancias
app-providers-no-longer-listed-by = { $m } ({ $name } ya no lo lista)
app-providers-now-lists-for-it-crow = { $m }: { $name } ahora indica { $addrs }; Crow sigue conectándose a la dirección antigua

## app-recovery
app-recovery-checking-the-host = comprobando el host…
app-recovery-connecting = conectando…
app-recovery-is-reachable-again = { $name } vuelve a estar accesible
app-recovery-type-exactly-to-trust-the = Escribe { $name } exactamente para confiar en la clave nueva.
app-recovery-no-key-was-fetched-from = No se obtuvo ninguna clave del servidor; no hay nada en lo que confiar.
app-recovery-known-hosts-wasnt-changed = known_hosts no se modificó: { $e }

## app-region
app-region-located = { $found } localizados
app-region-by-geoip = { $by_geoip } por GeoIP
app-region-without-cloud-metadata = { $unknown } sin metadatos de nube
app-region-unreachable = { $failed } inaccesibles
app-region-asking-server-s = consultando { $count } servidor(es)…
app-region-geoip-unavailable = { $note } · GeoIP no disponible: { $e }

## app-render
app-render-as = como { $name } · { $name2 } · { $session_label }
app-render-no-active-server-selected = NINGÚN SERVIDOR ACTIVO SELECCIONADO
app-render-select-a-server-tab-or = Selecciona la pestaña de un servidor o añade un host Linux nuevo para ver sus servicios y su configuración.
app-render-go-to-fleet-overview = ⬢ IR AL RESUMEN DE LA FLOTA
app-render-enroll-new-server = + AÑADIR SERVIDOR NUEVO
app-render-not-built-yet = { $active_view } · aún no está hecho
app-render-fleet = Flota
app-render-lab-container = contenedor de laboratorio
app-render-ssh-connecting = ssh · conectando
app-render-ssh-connected = ssh · conectado
app-render-ssh = ssh · { $label }

## app-rollouts
app-rollouts-the-machine-crow-runs-on = el equipo donde corre Crow: ejecútalo tú
app-rollouts-not-reachable-right-now = no accesible ahora mismo
app-rollouts-the-files-new-content = el contenido nuevo del archivo
app-rollouts-that-baselines-content-wasnt-kept = No se guardó el contenido de esa línea base (solo su hash), así que no se puede desplegar.
app-rollouts-write-the-command-to-run = Escribe primero el comando que hay que ejecutar.
app-rollouts-as-root = como root:{" "}
app-rollouts-rollout-command = DESPLIEGUE · COMANDO
app-rollouts-the-files-path-must-be = La ruta del archivo debe ser absoluta, como /etc/nginx/conf.d/app.conf.
app-rollouts-write-the-files-new-content = Escribe primero el contenido nuevo del archivo (o parte de la copia de un servidor o de una línea base).
app-rollouts-reading-each-servers-current-file = Leyendo el archivo actual de cada servidor para la vista previa; vuelve a planificar cuando se haya leído.
app-rollouts-already-identical = ya es idéntico
app-rollouts-write = escribir { $path } (+{ $add } −{ $del })
app-rollouts-create = crear { $path }
app-rollouts-couldnt-read-its-copy = no se pudo leer su copia: { $e }
app-rollouts-its-copy-hasnt-been-read = su copia aún no se ha leído
app-rollouts-rollout = DESPLIEGUE · { $path }
app-rollouts-remove-it-didnt-exist = eliminar { $path } (no existía)
app-rollouts-put-back-as-it-was = dejar { $path } como estaba
app-rollouts-roll-back = REVERTIR { $path } · { $n } { $n ->
        [one] SERVIDOR
       *[other] SERVIDORES
    }

## app-secrets
app-secrets-the-key-stored-there-is = la clave guardada ahí está dañada
app-secrets-not-saved-set-up-the = no guardado: configura primero la contraseña de la bóveda y vuelve a guardar
app-secrets-not-saved = no guardado
app-secrets-the-os-keyring-no-longer = El llavero del sistema ya no tenía la clave de cifrado de Crow, así que { $lost } secreto(s) guardado(s) no se pudieron leer y se borraron. Vuelve a introducirlos.
app-secrets-the-vault-is-busy = la bóveda está ocupada
app-secrets-moved-ai-api-key-s = Se movieron { $n } clave(s) de API de IA de texto plano a la bóveda cifrada.
app-secrets-couldnt-move-ai-api-keys = No se pudieron mover las claves de API de IA a la bóveda: { $e }
app-secrets-the-os-keyring-no-longer-2 = El llavero del sistema ya no tiene la clave de cifrado de Crow, así que no se pueden leer los tokens de proveedores ni las claves de IA guardados. Vuelve a introducirlos.

app-secrets-ai-api-keys-are-still = Las claves de API de IA siguen guardadas en texto plano: { $why }.
app-secrets-ai-api-keys-are-still-2 = Las claves de API de IA siguen guardadas en texto plano porque no hay llavero del sistema. Activa la contraseña de la bóveda para cifrarlas.

## app-server-facts
app-server-facts-reconnecting-to = Reconectando con { $name }…
app-server-facts-is-connected-again = { $name } vuelve a estar conectado
app-server-facts-couldnt-reconnect-to = No se pudo reconectar con { $name }: { $e }

## app-session
app-session-enter-master-password = Introduce la contraseña maestra…
app-session-password-cannot-be-empty = La contraseña no puede estar vacía
app-session-6-digit-2fa-code-is = Hace falta el código 2FA de 6 dígitos
app-session-this-computer = este ordenador
app-session-enter-master-password-min-8 = Introduce la contraseña maestra (mín. 8 caracteres)…
app-session-re-type-password = Repite la contraseña…
app-session-6-digit-code-e-g = código de 6 dígitos (p. ej. 123456)
app-session-password-must-be-at-least = La contraseña debe tener al menos 8 caracteres
app-session-passwords-do-not-match = Las contraseñas no coinciden
app-session-please-enter-the-6-digit = Introduce el código de confirmación de 6 dígitos de tu autenticador
app-session-invalid-6-digit-code-check = Código de 6 dígitos no válido. Revisa tu autenticador e inténtalo de nuevo

## app-settings
app-settings-not-changed = Sin cambios: { $message }
app-settings-not-changed-2 = Sin cambios: { $e }
app-settings-value = valor…

## app-team
app-team-not-allowed = No permitido: { $why }.
app-team-passphrase = frase de paso
app-team-name-e-g-ana = nombre, p. ej. ana
app-team-their-passphrase-8-characters-they = su frase de paso (8+ caracteres; puede cambiarla después)
app-team-a-second-members-passphrase = la frase de paso de un segundo miembro
app-team-signed-in-as = Sesión iniciada como { $name } ({ $name2 }).
app-team-that-passphrase-isnt-right = Esa frase de paso no es correcta.
app-team-give-the-member-a-name = Ponle al miembro un nombre que nadie más tenga.
app-team-team-mode-is-on-is = El modo equipo está activado: { $name } es el primer administrador y ha iniciado sesión. A partir de ahora Crow pregunta quién está al teclado al arrancar.
app-team-added-as = { $name } añadido como { $name2 }.
app-team-thats-the-last-admin-make = Es el último administrador: haz administrador a otra persona primero.
app-team-no-members-left-team-mode = No quedan miembros: el modo equipo está desactivado y Crow vuelve a ser de un solo usuario.
app-team-needs-a-second-members-approval = necesita la aprobación de un segundo miembro: otra persona con permiso para ejecuciones de flota escribe su frase de paso

## app-terminal
app-terminal-shell = shell { $i }
app-terminal-follows-the-shells-title-when = sigue el título de la shell si está vacío
app-terminal-couldnt-start-a-terminal = no se pudo abrir una terminal: { $e }

## app-updates
app-updates-downloading-and-verifying-crow = Descargando y verificando Crow { $version }…
app-updates-couldnt-start-the-new-version = No se pudo iniciar la versión nueva: { $e }

## app-users
app-users-not-applied = No aplicado: { $e }
app-users-ran = Ejecutado: { $summary }
app-users-failed = Falló: { $summary } — { $e }
app-users-not-applied-the-passwords-dont = No aplicado: las contraseñas no coinciden
app-users-new-password = contraseña nueva
app-users-again = otra vez
app-users-e-g-alice-deploy = p. ej. alicia, deploy
app-users-e-g-alice-wonderland-optional = p. ej. Alicia Maravillas (opcional)
app-users-password = contraseña
app-users-not-applied-crow-logs-in = No aplicado: Crow entra en este servidor como { $user }; eliminarlo dejaría fuera a Crow (y a ti, a través de Crow). Cambia antes el servidor a otra cuenta.
app-users-not-applied-thats-the-key = No aplicado: es la clave con la que Crow entra en este servidor; revocarla dejaría fuera a Crow. Cambia antes el servidor a otra clave (Claves → cambiar) y luego revoca esta.

## app-vault-manage
app-vault-manage-current-password = contraseña actual
app-vault-manage-2fa-code = código 2FA
app-vault-manage-new-password-8-characters = contraseña nueva (8+ caracteres)
app-vault-manage-new-password-again = repite la contraseña nueva
app-vault-manage-the-new-passwords-dont-match = las contraseñas nuevas no coinciden
app-vault-manage-password-changed-your-secrets-and = Contraseña cambiada. Tus secretos y tu 2FA no cambian.
app-vault-manage-tick-i-understand-first = Marca primero «Lo entiendo».
app-vault-manage-crow-is-open-now-no = Crow está ABIERTO ahora: sin contraseña, y el llavero del sistema guarda la clave.
app-vault-manage-still-locked-couldnt-hand-the = Sigue BLOQUEADO: no se pudo entregar la clave al llavero del sistema ({ $e }).

## about
about-debug-build = compilación de depuración
about-release-build = compilación de publicación
about-about-crow = Acerca de Crow
about-crow = Crow
about-native-linux-server-manager-over = Gestor nativo de servidores Linux por SSH
about-website = Web
about-source = Código
about-community = Comunidad
about-build = COMPILACIÓN
about-platform = Plataforma
about-profile = Perfil
about-ui-toolkit = Kit de interfaz
about-gpui-kit-0-6-1 = GPUI Kit 0.6.1 · WGPU
about-config-engine = Motor de configuración
about-crow-config-core-schema-ir = crow-config-core · schema IR
about-vault = Bóveda
about-argon2id-chacha20-poly1305-totp = Argon2id · ChaCha20-Poly1305 · TOTP
about-credits = CRÉDITOS
about-icons = Iconos
about-tabler-icons-mit = Tabler Icons · MIT
about-flags = Banderas
about-flag-icons-mit = flag-icons · MIT
about-distro-logos = Logos de distribuciones
about-dashboard-icons-by-homarr-labs = Dashboard Icons de homarr-labs · Apache-2.0
about-geoip-data = Datos GeoIP
about-font = Tipografía
about-jetbrains-mono-ofl-1-1 = JetBrains Mono · OFL-1.1
about-license = LICENCIA
about-aeupl-1-2-free-for = AEUPL-1.2. Gratis para cualquier uso no comercial. Consulta LICENSE en el repositorio.
about-copied-system-info = ✓ INFORMACIÓN DEL SISTEMA COPIADA
about-copy-system-info = COPIAR INFORMACIÓN DEL SISTEMA
about-ok = OK
about-pre-release = {" "}(PRELANZAMIENTO)
about-release-notes = NOTAS DE LA VERSIÓN
about-skip-this-version = OMITIR ESTA VERSIÓN
about-verify-install = VERIFICAR E INSTALAR
about-restart-now = REINICIAR AHORA
about-crow-doesnt-download-anything-unless = Crow no descarga nada a menos que actives la instalación en Ajustes → General.
about-crow-is-available = CROW { $version }{ $r } DISPONIBLE

## danger-zone
danger-zone-simulated-signal-sent-to-crow = Señal { $action } simulada enviada a { $name } — Crow todavía no ejecuta acciones de energía reales
danger-zone-this-host-isnt-running-ufw = Este host no está ejecutando ufw — no hay nada que vaciar.
danger-zone-ufw-rules-flushed-on = reglas de ufw vaciadas en { $label }
danger-zone-power = Energía
danger-zone-through-works-even-when-ssh = A través de { $name }: funciona aunque SSH no responda.
danger-zone-power-off = APAGAR
danger-zone-simulated-crow-doesnt-send-real = Simulado: Crow todavía no envía señales de energía reales a los hosts. Vincula un proveedor para controlar la energía de verdad.
danger-zone-reboot-45s-downtime = REINICIAR · 45 s SIN SERVICIO
danger-zone-snapshots = INSTANTÁNEAS…
danger-zone-snapshots-2 = Instantáneas

danger-zone-taken-and-listed-at = Se hacen y se listan en { $name }.
danger-zone-flush-firewall = Vaciar el cortafuegos
danger-zone-resets-ufw-every-rule-is = Reinicia ufw: se eliminan todas las reglas y el cortafuegos se desactiva.
danger-zone-flush-firewall-ufw = VACIAR CORTAFUEGOS (UFW)
danger-zone-rotate-host-keys = Rotar claves de host
danger-zone-not-built-yet = Aún no está hecho.
danger-zone-not-yet-built = AÚN NO ESTÁ HECHO
danger-zone-lab-containers = Contenedores de laboratorio
danger-zone-stops-every-lab-node-crow = Detiene todos los nodos de laboratorio que Crow creó en este equipo, y nada más.
danger-zone-kill-all-lab-containers = MATAR TODOS LOS CONTENEDORES DE LABORATORIO
danger-zone-archive-server = Archivar servidor
danger-zone-removes-it-from-the-fleet = Lo quita de la flota; sus datos se conservan hasta la fecha de eliminación y se puede restaurar.
danger-zone-archive-server-2 = ARCHIVAR SERVIDOR
danger-zone-danger-zone = ZONA DE PELIGRO
danger-zone-type-the-actions-keyword-to = escribe la palabra clave de la acción para confirmar · ninguna acción destructiva con un solo clic
danger-zone-type-to-confirm = Escribe { $keyword } para confirmar:
danger-zone-execute = EJECUTAR ⏎
danger-zone-cancel = cancelar

## distro-icon
distro-icon-alma-linux = alma linux
distro-icon-red-hat = red hat
distro-icon-arch-linux = arch linux
distro-icon-azure-linux = azure linux
distro-icon-mac-os = mac os
distro-icon-rhel = RHEL

## identity-bar
identity-bar-local = LOCAL
identity-bar-lab-container = CONTENEDOR DE LABORATORIO
identity-bar-connecting = CONECTANDO…
identity-bar-connected = CONECTADO
identity-bar-via-unreachable = mediante { $name } (inaccesible)
identity-bar-via = mediante { $name }
identity-bar-unverified = sin verificar
identity-bar-group = grupo
identity-bar-none = ninguno ▾
identity-bar-bastion = bastión ✓
identity-bar-bastion-no = bastión: no
identity-bar-search = Buscar
identity-bar-reconnect = RECONECTAR
identity-bar-start-vm = INICIAR VM
identity-bar-stop-vm = DETENER VM
identity-bar-restart-vm = REINICIAR VM
identity-bar-crow-reaches-through-which-is = Crow llega a { $name } a través de { $name2 }, que a su vez es inaccesible: arregla primero { $name2 }. { $detail }
identity-bar-crow-cant-read-right-now = Crow no puede leer { $name } ahora mismo — las páginas de abajo siguen vacías hasta que la conexión funcione. { $detail }
identity-bar-what-happened = qué pasó →
identity-bar-set-up-key-login = configurar acceso con clave →
identity-bar-region-unknown = región: desconocida ▾
identity-bar-region-unknown-2 = { $region_provider } · región desconocida ▾
identity-bar-set-by-hand = { $place } (a mano) ▾
identity-bar-by-ip = { $place } (por IP) ▾

## palette
palette-recent-first = { $scope_label } · recientes primero
palette-text = { $scope_label } · { $count } { $count ->
        [one] resultado
       *[other] resultados
    }
palette-nothing-matches-try-a-server = No hay coincidencias. Prueba con el nombre de un servidor, una página (logs, cortafuegos), un archivo de configuración o una acción.
palette-navigate = ↑↓ moverse
palette-open = ⏎ abrir
palette-into-a-server = ⇥ entrar en un servidor
palette-back-out = ⌫ volver
palette-esc-close = esc cerrar

## password-login
password-login-reading-sshds-settings = Leyendo los ajustes de sshd…
password-login-working-checking-a-fresh-key = Trabajando: comprobando un inicio de sesión nuevo con clave, escribiendo el archivo adicional, validando, recargando, volviendo a comprobar…
password-login-done-keep-a-key-for = Hecho. Guarda en un lugar seguro una clave para este servidor: las contraseñas ya no te dejarán entrar.
password-login-password-login-is-still-on = El inicio de sesión con contraseña sigue activado.
password-login-password-login-is-on-anyone = El inicio de sesión con contraseña está ACTIVADO. Quien adivine o robe una contraseña puede entrar, y los bots lo intentan sin parar. Si lo desactivas, solo funcionan las claves SSH.
password-login-what-crow-will-do-stopping = Lo que hará Crow, parando y deshaciendo al primer problema:
password-login-1-check-that-a-fresh = 1. Comprobar que funciona un inicio de sesión nuevo con clave (una conexión nueva, no la que mantiene Crow).
password-login-2-write-passwordauthentication-no-kbdinteractiveauthentication = 2. Escribir { $dropin } (PasswordAuthentication no, KbdInteractiveAuthentication no); se lee antes que los demás archivos adicionales, así que el ajuste propio de una imagen de nube no puede anularlo.
password-login-3-validate-with-sshd-t = 3. Validar con sshd -t y confirmar con sshd -T que las contraseñas están realmente desactivadas.
password-login-4-reload-sshd-open-sessions = 4. Recargar sshd (las sesiones abiertas siguen) y volver a comprobar un inicio de sesión nuevo con clave; si falla, borrar el archivo y recargar.
password-login-these-accounts-have-a-password = Estas cuentas tienen contraseña y perderán el acceso SSH con ella: { $pre }. Asegúrate de que cada una tenga antes una clave.
password-login-i-understand = Lo entiendo
password-login-turn-off-ssh-password-login = Desactivar el inicio de sesión SSH con contraseña · { $server_name }
password-login-turn-off-password-login = DESACTIVAR EL ACCESO CON CONTRASEÑA
password-login-cancel = CANCELAR
password-login-close = CERRAR

## recovery-panel
recovery-panel-checking = COMPROBANDO
recovery-panel-what-to-do = QUÉ HACER
recovery-panel-trust-new-key = CONFIAR EN LA CLAVE NUEVA
recovery-panel-install-key-with-password = INSTALAR CLAVE CON CONTRASEÑA
recovery-panel-try-again = REINTENTAR
recovery-panel-close = CERRAR

## sidebar
sidebar-overview = Resumen
sidebar-history = Historial
sidebar-services = Servicios
sidebar-processes = Procesos
sidebar-sockets = Sockets
sidebar-containers = Contenedores
sidebar-config = Configuración
sidebar-logs = Registros
sidebar-files = Archivos
sidebar-cron = Cron
sidebar-users = Usuarios
sidebar-firewall = Cortafuegos
sidebar-terminal = Terminal
sidebar-collapse = Contraer
sidebar-danger-zone = Zona de peligro
sidebar-plugins = PLUGINS

## snapshot-offer
snapshot-offer-snapshot-the-whole-server-at = ¿Hacer antes una instantánea del servidor entero en { $provider }?
snapshot-offer-youre-about-to-apply-a = Vas a aplicar { $file } ({ $description }). Un error aquí puede dejarte sin SSH, y entonces la copia de archivos de Crow no podrá llegar al servidor para deshacerlo. Una instantánea de { $provider } restaura el equipo entero desde fuera. El cambio espera hasta que { $provider } acepte la instantánea; si falla, no se aplica nada.
snapshot-offer-remember-my-choice-for-these = Recordar mi elección para estos cambios
snapshot-offer-taking-snapshot = HACIENDO INSTANTÁNEA…
snapshot-offer-snapshot-first-then-apply = PRIMERO INSTANTÁNEA, LUEGO APLICAR
snapshot-offer-apply-without = APLICAR SIN ELLA
snapshot-offer-cancel = CANCELAR

## snapshots-panel
snapshots-panel-asking = consultando a { $provider }…
snapshots-panel-has-no-snapshots-of-yet = { $provider } todavía no tiene instantáneas de { $server }.
snapshots-panel-automatic = (automática)
snapshots-panel-snapshots = INSTANTÁNEAS · { $server }
snapshots-panel-at = en { $provider }
snapshots-panel-restore-a-snapshot-from-the = Restaura una instantánea desde la consola del propio proveedor: restaurar sustituye los discos del servidor, y Crow no lo hace por ti.

## stance
stance-2fa = 2FA
stance-text = · { $weak } { $weak ->
        [one] problema
       *[other] problemas
    }
stance-security-stance = POSTURA DE SEGURIDAD
stance-no-vault-password-key-held = sin contraseña de bóveda · la clave la guarda el llavero del sistema
stance-vault-password-2fa-key-locked = contraseña de bóveda + 2FA · la clave la bloquea tu contraseña
stance-weakening-it-right-now = DEBILITÁNDOLA AHORA MISMO

stance-what-it-protects = QUÉ PROTEGE
stance-what-youre-exposed-to = A QUÉ TE EXPONES
stance-what-it-costs = QUÉ CUESTA
stance-nothing-is-usable-without-your = nada se puede usar sin tu contraseña y un código 2FA, a costa de desbloquear cada vez y de perder los secretos si pierdes cualquiera de los dos.
stance-no-unlocking-at-the-price = sin desbloquear, a costa de que cualquier cosa que se ejecute como tú pueda leer todos los secretos.
stance-switch-to-locked = CAMBIAR A BLOQUEADO →
stance-lock-now = BLOQUEAR AHORA
stance-vault-security-settings = AJUSTES DE BÓVEDA Y SEGURIDAD
stance-no-password = sin contraseña
stance-password-2fa = contraseña + 2FA
stance-protects = PROTEGE
stance-exposed-to = EXPUESTO A
stance-costs = CUESTA
stance-stay-open-and-save = SEGUIR ABIERTO Y GUARDAR
stance-i-understand-while-im-logged = Lo entiendo: mientras tenga la sesión iniciada, cualquier cosa que se ejecute como yo puede leer estos secretos.
stance-go-locked-set-a-password = PASAR A BLOQUEADO: PONER CONTRASEÑA →
stance-where-should-this-secret-stand = ¿Cómo debe quedar protegido este secreto?
stance-crow-is-about-to-store = Crow va a guardar su primer secreto. Las dos posturas lo cifran; se diferencian en quién puede llegar a él y en a qué renuncias. Elige una a propósito. Puedes cambiar a Bloqueado más tarde desde la insignia de postura.
stance-cancel-dont-save = CANCELAR (NO GUARDAR)
stance-the-other-stance = La otra postura, { $label }: { $other }

## stat-strip
stat-strip-vcpu-peak = { $vcpu_count } vCPU · pico { $m }%
stat-strip-inodes-iowait = inodos { $pct_or_dash } · iowait { $pct_or_dash2 }
stat-strip-mb-s = ↓ { $rx_mbps } · ↑ { $tx_mbps } Mb/s
stat-strip-turbo-buffer = BÚFER TURBO
stat-strip-s-lag-local-memory-cache = · -{ $lag_secs } s de retraso · caché en memoria local
stat-strip-ahead-in-s = A LA VISTA · { $description } · en { $lead_seconds } s
stat-strip-next-s-nothing-unusual = próximos { $lag_secs } s: nada inusual
stat-strip-cpu-load = CARGA CPU
stat-strip-memory = MEMORIA
stat-strip-disk-nvme = DISCO { $disk_mount } (NVME)
stat-strip-load-avg = CARGA MEDIA
stat-strip-uptime = TIEMPO ACTIVO
stat-strip-sampled = muestreado { $last_sample_ts }
stat-strip-network = RED
stat-strip-mb-s-2 = Mb/s

## status-bar
status-bar-no-open-alerts = ninguna alerta abierta
status-bar-1-open-alert = 1 alerta abierta
status-bar-open-alerts = { $n } alertas abiertas
status-bar-agentless-ssh = sin agente · ssh
status-bar-v-available = v{ $v } disponible ↗

## titlebar
titlebar-crow = CROW
titlebar-fleet = FLOTA
titlebar-agentless-ssh = { $server_count } { $server_count ->
        [one] host
       *[other] hosts
    } · ssh sin agente
titlebar-srv = SRV
titlebar-utc = UTC
titlebar-you = TÚ
titlebar-overview = { $name } · Resumen
titlebar-server-overview = Resumen del servidor
titlebar-fleet-overview = Resumen de la flota
titlebar-enroll-new-server = Añadir servidor nuevo…
titlebar-this-server = ESTE SERVIDOR
titlebar-managed-configs = Configuraciones gestionadas
titlebar-systemd-journal-logs = Registros del journal de systemd
titlebar-application = APLICACIÓN
titlebar-command-palette = Paleta de comandos…
titlebar-settings = Ajustes
titlebar-about-crow = Acerca de Crow…
titlebar-session = SESIÓN
titlebar-lock-vault-disconnect = Bloquear bóveda y desconectar
titlebar-quit-crow = Salir de Crow

## audit-model
audit-model-by = { $text } · por { $a }
audit-model-enrolled = añadido { $login_user }@{ $host }:{ $port }
audit-model-saved = guardado { $target }
audit-model-restored-an-earlier-version-of = restaurada una versión anterior de { $target }
audit-model-baseline-for = línea base de { $target }: { $before_state }
audit-model-killed-process = proceso { $target } terminado
audit-model-purged-stored-data-for = datos guardados de { $target } eliminados
audit-model-turned-off-ssh-password-login = desactivado el inicio de sesión SSH con contraseña
audit-model-trusted-a-new-host-key = confiada una clave de host nueva para { $target }
audit-model-opened-a-terminal = abrió una terminal
audit-model-at-the-provider = { $k } en el proveedor
audit-model-rolling-reboot = reinicio escalonado: { $before_state }
audit-model-ufw = ufw { $target }
audit-model-user = usuario { $target }

## audit
audit-audit-log = REGISTRO DE AUDITORÍA
audit-failed-every-server = { $count } { $count ->
        [one] cambio
       *[other] cambios
    } · { $failed } fallidos · todos los servidores
audit-server = SERVIDOR
audit-kind = TIPO
audit-nothing-recorded-yet = Todavía no hay nada registrado.

## config-cron-editor
config-cron-editor-every-minute = Cada minuto (* * * * *)
config-cron-editor-every-minutes = Cada { $step } minutos
config-cron-editor-every-hour-on-the-hour = Cada hora en punto (0 * * * *)
config-cron-editor-every-hour-at-minute = Cada hora en el minuto { $m }
config-cron-editor-every-day-at = Cada día a las { $h }:{ $m }
config-cron-editor-sunday = domingo
config-cron-editor-monday = lunes
config-cron-editor-tuesday = martes
config-cron-editor-wednesday = miércoles
config-cron-editor-thursday = jueves
config-cron-editor-friday = viernes
config-cron-editor-saturday = sábado
config-cron-editor-weekday-mon-fri = Laborables (lun-vie)
config-cron-editor-weekend-sat-sun = Fin de semana (sáb-dom)
config-cron-editor-weekly-on-at = Cada semana el { $day_name } a las { $h }:{ $m }
config-cron-editor-monthly-on-day-at = Cada mes el día { $dom } a las { $h }:{ $m }
config-cron-editor-schedule = Programación: { $m } { $h } { $dom } { $mon } { $dow }
config-cron-editor-active-schedules = { $active_count } PROGRAMACIONES ACTIVAS
config-cron-editor-unsaved-edits = CAMBIOS SIN GUARDAR
config-cron-editor-in-sync = SINCRONIZADO
config-cron-editor-new-cron-job = + NUEVA TAREA CRON
config-cron-editor-revert = DESHACER
config-cron-editor-stage-apply = PREPARAR Y APLICAR
config-cron-editor-history = HISTORIAL · { $rev_count }
config-cron-editor-each-job-runs-as-its = Cada tarea se ejecuta como su USUARIO · cron vuelve a leer /etc/crontab por sí solo tras guardar
config-cron-editor-no-jobs-in-etc-crontab = No hay tareas en /etc/crontab. + NUEVA TAREA CRON añade una.
config-cron-editor-on = ACTIVA
config-cron-editor-schedule-2 = PROGRAMACIÓN
config-cron-editor-user = USUARIO
config-cron-editor-command = COMANDO
config-cron-editor-every-15m = Cada 15 min
config-cron-editor-hourly = Cada hora
config-cron-editor-daily-3-am = Diaria (3:00)
config-cron-editor-daily-midnight = Diaria (medianoche)
config-cron-editor-weekly-sun = Semanal (dom)
config-cron-editor-monthly-1st = Mensual (día 1)
config-cron-editor-workdays-9-am = Laborables (9:00)
config-cron-editor-quick-schedule-presets = PROGRAMACIONES RÁPIDAS
config-cron-editor-5-field-schedule-expression = EXPRESIÓN DE PROGRAMACIÓN DE 5 CAMPOS
config-cron-editor-minute-0-59 = Minuto (0-59)
config-cron-editor-hour-0-23 = Hora (0-23)
config-cron-editor-day-of-month-1-31 = Día del mes (1-31)
config-cron-editor-month-1-12 = Mes (1-12)
config-cron-editor-day-of-week-0-7 = Día de la semana (0-7)
config-cron-editor-execution-user = USUARIO DE EJECUCIÓN
config-cron-editor-shell-command = COMANDO DE SHELL
config-cron-editor-any = {"*"} (cualquiera)

## config-journald-editor
config-journald-editor-crash-safe = A PRUEBA DE CAÍDAS
config-journald-editor-volatile-risk = RIESGO VOLÁTIL
config-journald-editor-unsaved-edits = CAMBIOS SIN GUARDAR
config-journald-editor-in-sync = SINCRONIZADO

config-journald-editor-usage = Uso: { $disk_usage_display } / { $max_use_display }
config-journald-editor-revert = DESHACER
config-journald-editor-stage-apply-config = PREPARAR Y APLICAR CONFIG.
config-journald-editor-history = HISTORIAL · { $rev_count }
config-journald-editor-why-default-linux-journal-configs = POR QUÉ LA CONFIGURACIÓN DEL JOURNAL POR DEFECTO FALLA EN PRODUCCIÓN
config-journald-editor-by-default-systemd-journal-uses = Por defecto, el journal de systemd usa límites silenciosos fijados al compilar y Storage=auto. En servidores con mucha actividad o en reinicios tras un incidente, los registros rotan en 30 minutos o desaparecen del todo. Crow-Config impone límites explícitos para que nunca pierdas las pruebas de una caída.
config-journald-editor-storage = Almacenamiento
config-journald-editor-controls-where-journal-files-are = Controla dónde se guardan los archivos del journal. Volatile los mantiene en RAM (/run), y se borran para siempre al reiniciar. Persistent garantiza que se escriben en /var/log/journal/.
config-journald-editor-persistent-rec = PERSISTENTE (Rec.)
config-journald-editor-survives-reboots = Sobrevive a los reinicios
config-journald-editor-volatile = VOLÁTIL
config-journald-editor-ram-only-wiped-on-reboot = Solo RAM · Se borra al reiniciar
config-journald-editor-auto = AUTO
config-journald-editor-default-fallback = Opción por defecto
config-journald-editor-systemmaxuse = SystemMaxUse
config-journald-editor-the-maximum-disk-space-journals = El espacio máximo en disco que pueden ocupar los journals en almacenamiento persistente. Al superarlo, systemd elimina los archivos más antiguos.
config-journald-editor-4g-rec = 4G (Rec.)
config-journald-editor-custom = Personalizado
config-journald-editor-maxretentionsec = MaxRetentionSec
config-journald-editor-the-maximum-time-to-store = El tiempo máximo que se guardan las entradas del journal. Evita que los archivos antiguos se queden indefinidamente cuando hay pocos registros.
config-journald-editor-30day-rec = 30day (Rec.)
config-journald-editor-0-unlimited = 0 (Ilimitado)
config-journald-editor-systemkeepfree = SystemKeepFree
config-journald-editor-guarantees-that-at-least-this = Garantiza que quede libre al menos este espacio en el sistema de archivos raíz, para evitar caídas por disco lleno debidas a una avalancha de registros.
config-journald-editor-1-0-gb = 1,0 GB
config-journald-editor-2-0-gb-rec = 2,0 GB (Rec.)
config-journald-editor-5-0-gb = 5,0 GB
config-journald-editor-ratelimitburst-ratelimitintervalsec = RateLimitBurst / RateLimitIntervalSec
config-journald-editor-suppresses-runaway-spam-loops-from = Evita que bucles de mensajes descontrolados inunden los journals y ahoguen el sistema (10.000 entradas cada 30 segundos por defecto).
config-journald-editor-entries-s = { $rate_limit_burst } entradas / { $rate_limit_interval_sec } s
config-journald-editor-anti-flood-enabled = ✓ Protección contra avalanchas activada

## config-managed-files
config-managed-files-config-files = ARCHIVOS DE CONFIGURACIÓN
config-managed-files-unverified-distro-generic-scan-only = ⚠ Distribución sin verificar — solo análisis genérico, pueden faltar configuraciones.
config-managed-files-crow-ui-mapped = ● CON INTERFAZ DE CROW
config-managed-files-edited = EDITADO
config-managed-files-crow-ui = INTERFAZ CROW
config-managed-files-system-configs-raw = ○ CONFIGURACIONES DEL SISTEMA (TEXTO)
config-managed-files-ro = SL
config-managed-files-raw = TEXTO
config-managed-files-schema-packs-active = PAQUETES DE ESQUEMA ACTIVOS
config-managed-files-systemd-postgres-openssh-ufw = systemd · postgres · openssh · ufw

## config-pending-diff-rail
config-pending-diff-rail-save-failed = ERROR AL GUARDAR · { $err }
config-pending-diff-rail-read-only = SOLO LECTURA · { $reason }
config-pending-diff-rail-history = HISTORIAL · { $err }
config-pending-diff-rail-restore-v = ¿RESTAURAR v{ $v }?
config-pending-diff-rail-same-as-host = igual que en el host
config-pending-diff-rail-pending-diff = CAMBIOS PENDIENTES
config-pending-diff-rail-unified = unificado · { $sel_file }
config-pending-diff-rail-author = Autor:
config-pending-diff-rail-on-host = --- en el host/{ $sel_file }
config-pending-diff-rail-back-to-pending-diff = ← volver a los cambios pendientes
config-pending-diff-rail-baseline = --- línea base/{ $sel_file }
config-pending-diff-rail-staged-unsaved = +++ preparado/{ $sel_file } (sin guardar)
config-pending-diff-rail-no-unsaved-edits = No hay cambios sin guardar
config-pending-diff-rail-on-save = AL GUARDAR
config-pending-diff-rail-version-history-revisions = HISTORIAL DE VERSIONES ({ $count } REVISIONES)
config-pending-diff-rail-on-host-not-recorded = en el host: no registrado
config-pending-diff-rail-on-host-v = En el host: v{ $active_rev }
config-pending-diff-rail-hash-only = SOLO HASH
config-pending-diff-rail-restore = RESTAURAR
config-pending-diff-rail-current = ACTUAL
config-pending-diff-rail-by = por
config-pending-diff-rail-baseline-2 = LÍNEA BASE
config-pending-diff-rail-pin-as-baseline = FIJAR COMO LÍNEA BASE
config-pending-diff-rail-revert = DESHACER
config-pending-diff-rail-stage-apply = PREPARAR Y APLICAR
config-pending-diff-rail-written-atomically-temp-file-then = Escritura atómica: archivo temporal y luego renombrado
config-pending-diff-rail-structured-edits-keep-comments-and = Los cambios estructurados conservan comentarios y formato
config-pending-diff-rail-no-host-validator-for-this = No hay validador en el host para este formato
config-pending-diff-rail-checked-on-the-host-first = Se comprueba antes en el host: { $cmds }
config-pending-diff-rail-plain-text-nothing-checks-it = Texto plano: nada lo comprueba antes de escribirlo
config-pending-diff-rail-not-kept-in-history-crow = No se guarda en el historial: Crow no leyó este archivo del host
config-pending-diff-rail-every-version-kept-in-history = Cada versión se guarda cifrada en el historial
config-pending-diff-rail-history-keeps-hashes-only-no = El historial guarda solo hashes: no hay clave de cifrado disponible
config-pending-diff-rail-history-not-loaded-yet = Historial aún no cargado
config-pending-diff-rail-none-pinned-pin-as-baseline = Ninguna fijada. FIJAR COMO LÍNEA BASE en una revisión la convierte en el { $filename } correcto de { $scope_label }; los servidores que difieran aparecen como desviados.
config-pending-diff-rail-matches-the-baseline = Coincide con la línea base
config-pending-diff-rail-same-settings-as-the-baseline = Mismos ajustes que la línea base; solo cambian comentarios o formato
config-pending-diff-rail-drifted-from-the-baseline = Desviado de la línea base
config-pending-diff-rail-more = … { $more } más
config-pending-diff-rail-pinned-from-for-by = fijada desde { $from_server } para { $scope_label } por { $set_by }
config-pending-diff-rail-load-baseline-into-editor = CARGAR LÍNEA BASE EN EL EDITOR
config-pending-diff-rail-unpin = LIBERAR

## config-raw-editor
config-raw-editor-traditional-editor = EDITOR TRADICIONAL
config-raw-editor-edited = EDITADO (+{ $add_count } -{ $del_count })
config-raw-editor-v-baseline = v{ $active_revision } (LÍNEA BASE)
config-raw-editor-author = Autor:
config-raw-editor-history = HISTORIAL · { $count }
config-raw-editor-structured-view = VISTA ESTRUCTURADA
config-raw-editor-revert-baseline = VOLVER A LA LÍNEA BASE
config-raw-editor-stage-version = PREPARAR VERSIÓN
config-raw-editor-lines = { $line_count } líneas
config-raw-editor-bytes = { $count } bytes
config-raw-editor-utf-8 = UTF-8
config-raw-editor-unix-lf = UNIX (LF)
config-raw-editor-revisions = Revisiones: { $count }

config-raw-editor-read-only = SOLO LECTURA
config-raw-editor-unsaved-staged-changes = ● Cambios preparados sin guardar
config-raw-editor-clean-baseline = ✓ Línea base limpia

## config-sshd-sheet
config-sshd-sheet-some-files-this-config-includes = No se pudieron leer algunos archivos que incluye esta configuración, así que los valores de abajo pueden no ser los que usa sshd:
config-sshd-sheet-other-directives = OTRAS DIRECTIVAS
config-sshd-sheet-match-in = MATCH { $scope } · EN { $path }
config-sshd-sheet-match-applies-only-when-this = MATCH { $scope } · SOLO SE APLICA CUANDO COINCIDE
config-sshd-sheet-nothing-to-fix = nada que corregir
config-sshd-sheet-1-to-fix = 1 por corregir
config-sshd-sheet-to-fix = { $n } por corregir
config-sshd-sheet-security-posture = POSTURA DE SEGURIDAD
config-sshd-sheet-fine = { $count } correctos
config-sshd-sheet-set-in = definido en { $src }
config-sshd-sheet-sshds-default = valor por defecto de sshd
config-sshd-sheet-set = PONER { $v }
config-sshd-sheet-turn-off-safely = DESACTIVAR CON SEGURIDAD…
config-sshd-sheet-not-set = sin definir
config-sshd-sheet-line = línea { $l }
config-sshd-sheet-ignored-sets-it-first = { $at } · ignorado: { $by } lo define antes
config-sshd-sheet-ignored-set-earlier = { $at } · ignorado, definido antes
config-sshd-sheet-back-to-sshds-default-removes = Volver al valor por defecto de sshd (quita la línea)
config-sshd-sheet-remove-the-line = Quitar la línea
config-sshd-sheet-default = por defecto

## config-state
config-state-differs-from-the-baseline-only = difiere de la línea base (solo se guardó su hash)
config-state-was-not-found-on-this = { $file } no se encontró en este servidor
config-state-new-job = Tarea nueva

## config-structured-editor
config-structured-editor-recommended = RECOMENDADO
config-structured-editor-caution = PRECAUCIÓN
config-structured-editor-weak = DÉBIL
config-structured-editor-never-on-prod = NUNCA EN PRODUCCIÓN
config-structured-editor-deny = DENEGAR
config-structured-editor-lets-run = permite que { $field } ejecute { $risk }
config-structured-editor-crow-ui = INTERFAZ CROW · { $plugin_name } · { $kind }
config-structured-editor-edited = EDITADO (+{ $add_count } -{ $del_count })
config-structured-editor-v-baseline = v{ $active_revision } (LÍNEA BASE)
config-structured-editor-text-view = VISTA DE TEXTO
config-structured-editor-history = HISTORIAL
config-structured-editor-revert = DESHACER
config-structured-editor-stage-version = PREPARAR VERSIÓN
config-structured-editor-rows-are-evaluated-top-to = las filas se evalúan de arriba abajo
config-structured-editor-1-value-here-isnt = 1 valor de aquí no es
config-structured-editor-values-here-arent = { $not_running } valores de aquí no son
config-structured-editor-not-running-yet-what-the = AÚN NO EN USO · { $what } lo que usa el kernel ahora; se aplican al arrancar o con sysctl --system
config-structured-editor-managed-by-it-rewrites-this = GESTIONADO POR { $m } · reescribe este archivo, así que los cambios aquí se pierden; cambia los ajustes de DNS en { $m2 }
config-structured-editor-read-only = SOLO LECTURA · { $r }
config-structured-editor-edit-rejected = CAMBIO RECHAZADO · { $e }
config-structured-editor-line = LÍNEA
config-structured-editor-add-directive = + AÑADIR DIRECTIVA
config-structured-editor-no-entries-only-comments-or = Sin entradas — solo comentarios o líneas en blanco.
config-structured-editor-move-up-rules-are-read = Subir: las reglas se leen de arriba abajo
config-structured-editor-scoped-to-match-read-only = Limitado a Match { $s } — solo lectura hasta que crow-config modele los bloques Match (ERR-12)
config-structured-editor-add-row = + AÑADIR FILA
config-structured-editor-add = AÑADIR
config-structured-editor-added-at-the-end-of = Se añade al final de este bloque. Los bloques nuevos se escriben en la vista de texto.
config-structured-editor-added-at-the-end-of-2 = Se añade al final del archivo.
config-structured-editor-added-at-the-end-of-3 = Se añade al final de esta sección. Las secciones nuevas se escriben en la vista de texto.
config-structured-editor-this-change-introduces-values-rated = Este cambio introduce valores marcados como NUNCA EN PRODUCCIÓN:
config-structured-editor-type-to-stage-it = Escribe { $risk_confirm_keyword } para prepararlo:
config-structured-editor-stage-anyway = PREPARAR DE TODOS MODOS
config-structured-editor-cancel = CANCELAR
config-structured-editor-order-matters = EL ORDEN IMPORTA · { $ir }

## containers
containers-running = Ejecutando: { $b }…
containers-reading-the-servers-containers = Leyendo los contenedores del servidor…
containers-couldnt-read-the-server = No se pudo leer el servidor: { $e }
containers-containers = CONTENEDORES · { $server_name }
containers-running-2 = { $running }/{ $count } EN MARCHA
containers-reading = LEYENDO…
containers-refresh = ACTUALIZAR
containers-fix-it-on-the-server = Arréglalo en el servidor
containers-neither-docker-nor-podman-is = Ni Docker ni Podman están instalados en este servidor.
containers-no-container-runtime-crow-can = Aquí no hay ningún entorno de contenedores que Crow pueda usar.
containers-no-containers-on-this-server = No hay contenedores en este servidor.
containers-standalone = SUELTOS
containers-not-in-a-compose-stack = { $count } fuera de una pila compose
containers-up = LEVANTAR
containers-restart = REINICIAR
containers-down = BAJAR
containers-running-3 = { $running }/{ $count } en marcha
containers-create-and-start-every-service = ¿Crear e iniciar todos los servicios de esta pila (puede descargar imágenes)?
containers-restart-every-container-in-this = ¿Reiniciar todos los contenedores de esta pila? Cada uno se detiene un momento.
containers-stop-and-remove-this-stacks = ¿Detener y eliminar los contenedores y redes de esta pila? Los volúmenes con nombre se conservan.
containers-cancel = CANCELAR
containers-name = NOMBRE
containers-image = IMAGEN
containers-state = ESTADO
containers-published-ports = PUERTOS PUBLICADOS
containers-restarts = REINICIOS
containers-runtime = ENTORNO
containers-all-nets = TODAS LAS REDES
containers-root = {" "}· root
containers-no-output-yet = Todavía no hay salida.
containers-reading-logs = Leyendo registros…
containers-reading-2 = Leyendo…
containers-start = INICIAR
containers-stop = DETENER
containers-start-this-container = ¿Iniciar este contenedor?
containers-stop-this-container-what-it = ¿Detener este contenedor? Lo que sirve deja de estar disponible hasta que se vuelva a iniciar.
containers-restart-this-container-its-down = ¿Reiniciar este contenedor? Se detiene un momento.
containers-logs = REGISTROS
containers-inspect = INSPECCIONAR
containers-newest-first = más recientes primero
containers-following = SIGUIENDO
containers-follow = SEGUIR
containers-environment-values-hidden = valores del entorno ocultos

## files-collector
files-collector-binary-file-nothing-to-show = archivo binario: no hay nada que mostrar como texto
files-collector-invalid-folder-name = Nombre de carpeta no válido

## files-models
files-models-kb = { $size_bytes } KB
files-models-mb = { $size_bytes } MB
files-models-gb = { $size_bytes } GB

## files
files-files = ARCHIVOS
files-up = SUBIR
files-folder = + CARPETA
files-name = NOMBRE
files-size = TAMAÑO
files-perms = PERMISOS
files-owner = PROPIETARIO
files-group = GRUPO
files-modified = MODIFICADO
files-empty-directory = Carpeta vacía
files-create = CREAR ⏎
files-cancel = cancelar
files-edit-in-a-terminal-nano = Editar en una terminal (nano)
files-delete-empty-directory-this-cannot = ¿Eliminar la carpeta vacía { $del_name }? No se puede deshacer.

files-delete-this-cannot-be-undone = ¿Eliminar { $del_name }? No se puede deshacer.
files-cancel-2 = Cancelar
files-delete = ELIMINAR ⏎
files-others = OTROS
files-everyone-else = todos los demás
files-setuid = SETUID
files-runs-as-its-owner = se ejecuta como su propietario
files-setgid = SETGID
files-new-files-get-this-group = los archivos nuevos reciben este grupo
files-runs-as-its-group = se ejecuta como su grupo
files-sticky = STICKY
files-only-owners-delete-inside = dentro solo borran los propietarios
files-anyone-on-the-server-can = cualquiera en el servidor puede cambiar este archivo
files-applying = aplicando…
files-chmod = chmod { $mode }
files-reading = leyendo…
files-first-kb-lines = primeros { $collector } KB · { $lines } líneas
files-text = { $lines } { $lines ->
        [one] línea
       *[other] líneas
    }
files-edit-in-terminal = EDITAR EN TERMINAL
files-reading-the-file = Leyendo el archivo…

## firewall-detector
firewall-detector-ufw-is-active-but-its = ufw está activo pero no se pudo interpretar su salida de estado.
firewall-detector-ufw-is-installed-but-inactive = UFW está instalado pero inactivo. El filtrado de paquetes netfilter del sistema está desactivado.
firewall-detector-ufw-is-installed-but-crow = ufw está instalado, pero Crow no pudo leer sus reglas como root: { $e }
firewall-detector-firewalld-is-installed-but-not = firewalld está instalado pero no está en marcha, así que no filtra nada.
firewall-detector-firewalld-is-running-but-crow = firewalld está en marcha, pero Crow no pudo leer sus zonas como root: { $e }
firewall-detector-nftables-iptables-are-here-but = nftables/iptables están aquí, pero Crow no pudo leer el conjunto de reglas como root: { $e }
firewall-detector-no-firewall-tooling-ufw-firewalld = No se encontró en este host ninguna herramienta de cortafuegos (ufw, firewalld, nftables, iptables).

## firewalld
firewalld-from = desde { $z }
firewalld-on = en { $z }
firewalld-every-interface-not-in-another = todas las interfaces que no están en otra zona
firewalld-zone-accepts-all-traffic = la zona { $name } acepta todo el tráfico ({ $binding })
firewalld-service-ports-not-known-zone = servicio { $svc } (puertos desconocidos) · zona { $name }
firewalld-service-zone = servicio { $svc } · zona { $name }
firewalld-port-zone = puerto · zona { $name }
firewalld-rich-rule-zone = regla rica · zona { $name }
firewalld-rich-rule-service-zone = regla rica, servicio { $svc } · zona { $name }
firewalld-rich-rule-not-interpreted = regla rica (sin interpretar): { $raw_rule }
firewalld-the-running-firewall-differs-from = El cortafuegos en marcha difiere de su configuración guardada (permanente): una recarga o un reinicio perderá los cambios hechos solo en caliente.

## firewall-lockout
firewall-lockout-crow-cant-see-which-rules = Crow no puede ver qué reglas se aplicarán después, así que no puede comprobar que el puerto { $port }/tcp, el puerto SSH por el que está conectado, siga permitido.
firewall-lockout-after-this-port-tcp-crows = Después de esto, el puerto { $port }/tcp (la conexión SSH de Crow) solo se permite desde { $value }. Si Crow no se conecta desde ahí, el siguiente inicio de sesión se rechaza.
firewall-lockout-this-stops-port-tcp-the = Esto deja de permitir el puerto { $port }/tcp, el puerto SSH por el que está conectado Crow. El siguiente inicio de sesión SSH, incluido el de Crow, se rechazaría.

## firewall-models
firewall-models-ufw-uncomplicated-firewall = UFW (Uncomplicated Firewall)
firewall-models-unmanaged-none-detected = Sin gestionar / Ninguno detectado
firewall-models-in = ENT
firewall-models-out = SAL
firewall-models-fwd = REENV
firewall-models-allow = PERMITIR
firewall-models-deny = DENEGAR
firewall-models-reject = RECHAZAR
firewall-models-limit = LIMITAR
firewall-models-tcp = TCP
firewall-models-udp = UDP
firewall-models-tcp-udp = TCP+UDP
firewall-models-any = CUALQUIERA
firewall-models-tcp-udp-2 = { $port }/tcp,udp
firewall-models-firewall-allowed = CORTAFUEGOS: PERMITIDO
firewall-models-firewall-blocked = CORTAFUEGOS: BLOQUEADO
firewall-models-firewall-allow-default = CORTAFUEGOS: PERMITIDO (POR DEFECTO)
firewall-models-firewall-no-inbound-rule = CORTAFUEGOS: SIN REGLA DE ENTRADA
firewall-models-firewall-inactive = CORTAFUEGOS: INACTIVO
firewall-models-firewall-not-read = CORTAFUEGOS: SIN LEER

## firewall
firewall-firewall-is-currently-disabled-by = El administrador ha desactivado el cortafuegos.
firewall-all = TODAS
firewall-allow = PERMITIR
firewall-deny = DENEGAR
firewall-limit = LIMITAR
firewall-firewall = CORTAFUEGOS
firewall-on = { $short_name } · activo · { $total_rules } { $total_rules ->
        [one] regla
       *[other] reglas
    }
firewall-stage = PREPARAR +{ $add_count } −{ $del_count }
firewall-staged-firewall-rule-changes = Cambios de reglas del cortafuegos preparados
firewall-audit = AUDITORÍA
firewall-rule-table = TABLA DE REGLAS
firewall-raw-ruleset = REGLAS EN BRUTO
firewall-reload = RECARGAR
firewall-disable = DESACTIVAR
firewall-add-rule = + AÑADIR REGLA
firewall-defaults = POR DEFECTO
firewall-quick-open = APERTURA RÁPIDA
firewall-action = ACCIÓN
firewall-dir = DIR
firewall-port = PUERTO
firewall-from = DESDE
firewall-to = HACIA
firewall-comment = COMENTARIO
firewall-no-rules-yet-everything-follows = Todavía no hay reglas: todo sigue los valores por defecto de arriba
firewall-no-rules-match-try-all = Ninguna regla coincide: prueba con TODAS o borra la búsqueda
firewall-running = Ejecutando: { $p } …
firewall-read-only-crow-reads-the = Solo lectura: Crow lee las reglas de { $label } (la cadena de entrada y a dónde salta) pero nunca escribe reglas en bruto.
firewall-read-only-crow-reads-the-2 = Solo lectura: Crow lee { $label } (la configuración en marcha) pero aún no la cambia; usa sus propias herramientas para editarla.
firewall-changes-go-to-firewallds-running = Los cambios van a la vez a la configuración en marcha de firewalld y a la guardada (permanente), así que se aplican ya y sobreviven a una recarga.
firewall-this-could-lock-you-out = ESTO PODRÍA DEJARTE FUERA
firewall-type-confirm-to-apply-it = Escribe CONFIRM para aplicarlo de todos modos:
firewall-apply-anyway = APLICAR DE TODOS MODOS
firewall-cancel = CANCELAR

## firewall-new-rule-modal
firewall-new-rule-modal-anywhere = Cualquier origen
firewall-new-rule-modal-will-run-running-saved-config = SE EJECUTARÁ (configuración en marcha + guardada)
firewall-new-rule-modal-not-valid-for-firewalld = NO VÁLIDO PARA FIREWALLD
firewall-new-rule-modal-will-run = SE EJECUTARÁ
firewall-new-rule-modal-add-rule = AÑADIR REGLA
firewall-new-rule-modal-action = ACCIÓN
firewall-new-rule-modal-allow = PERMITIR
firewall-new-rule-modal-deny = DENEGAR
firewall-new-rule-modal-reject = RECHAZAR
firewall-new-rule-modal-limit = LIMITAR
firewall-new-rule-modal-protocol = PROTOCOLO
firewall-new-rule-modal-tcp = TCP
firewall-new-rule-modal-udp = UDP
firewall-new-rule-modal-tcp-udp = TCP+UDP
firewall-new-rule-modal-any = CUALQUIERA
firewall-new-rule-modal-port = PUERTO
firewall-new-rule-modal-from-2 = DESDE
firewall-new-rule-modal-comment-2 = COMENTARIO
firewall-new-rule-modal-enter-adds-it-esc-or = Intro la añade · Esc o un clic fuera cancela
firewall-new-rule-modal-cancel = CANCELAR

## firewall-non-operational
firewall-non-operational-no-firewall-running = NINGÚN CORTAFUEGOS EN MARCHA
firewall-non-operational-why = POR QUÉ
firewall-non-operational-firewall-tools-on-the-host = HERRAMIENTAS DE CORTAFUEGOS EN EL HOST
firewall-non-operational-no-standard-packet-filter-binaries = No se encontraron binarios estándar de filtrado de paquetes (ufw, firewalld, iptables)
firewall-non-operational-to-turn-it-on = PARA ACTIVARLO:

firewall-non-operational-to-set-one-up-ufw = PARA CONFIGURAR UNO (ufw):
firewall-non-operational-turn-the-firewall-on = ACTIVAR EL CORTAFUEGOS

## fleet-archived
fleet-archived-active-fleet = FLOTA ACTIVA ({ $active_n })
fleet-archived-archived = ARCHIVADOS ({ $archived_n })
fleet-archived-danger-zone = ZONA DE PELIGRO
fleet-archived-host = HOST
fleet-archived-address = DIRECCIÓN
fleet-archived-env = ENTORNO
fleet-archived-role = ROL
fleet-archived-archived-2 = ARCHIVADO
fleet-archived-stored-data = DATOS GUARDADOS
fleet-archived-restore = RESTAURAR
fleet-archived-no-archived-servers = NO HAY SERVIDORES ARCHIVADOS
fleet-archived-archiving-takes-a-server-out = Archivar saca un servidor de la flota sin borrarlo. Nunca se borra nada de golpe.
fleet-archived-purge-log = REGISTRO DE ELIMINACIONES
fleet-archived-record-s-crows-own-audit = { $count } registro(s) · el propio rastro de auditoría de Crow
fleet-archived-nothing-has-been-purged-yet = Todavía no se ha eliminado nada.
fleet-archived-archive = ARCHIVAR { $name }
fleet-archived-leaves-the-active-fleet-crow = { $name } sale de la flota activa. Crow deja de consultarlo y cierra su conexión SSH.
fleet-archived-its-stored-data-change-history = Sus datos guardados — historial de cambios y claves vinculadas — se eliminan el { $purge_due }.
fleet-archived-the-server-record-itself-is = El registro del servidor nunca se elimina: dirección, puerto, usuario, clave, host de salto, entorno, rol, grupo, etiquetas, huella de la clave de host y datos del sistema se conservan. Se puede restaurar desde la pestaña Archivados hasta que se ejecute la eliminación; después vuelve sin su historial.
fleet-archived-cancel = CANCELAR
fleet-archived-archive-server = ARCHIVAR SERVIDOR

## fleet-certificates
fleet-certificates-certificates = CERTIFICADOS
fleet-certificates-served-expiring-within-days-not = { $total } en uso · { $soon } caducan en { $cert_warn_days } días · { $broken } { $broken ->
        [one] servidor no se renueva
       *[other] servidores no se renuevan
    } solo
fleet-certificates-read-again = LEER DE NUEVO
fleet-certificates-reading = LEYENDO { $count }…
fleet-certificates-renew-runs-certbots-dry-run = RENOVAR ejecuta primero la prueba en seco de certbot, luego la renovación, después comprueba y recarga nginx o Apache (nunca con una configuración rota) y vuelve a leer el certificado para confirmar su nueva caducidad. certbot solo renueva a menos de 30 días de la caducidad; Let's Encrypt limita cuántas veces se puede emitir un certificado.
fleet-certificates-no-renewal-tool = sin herramienta de renovación
fleet-certificates-couldnt-read = no se pudo leer: { $e }
fleet-certificates-reading-2 = leyendo…
fleet-certificates-renews-on-its-own = se renueva solo
fleet-certificates-no-certificates-found-lets-encrypts = No se encontraron certificados (carpeta live de Let's Encrypt, configuraciones de nginx y Apache).
fleet-certificates-no-name = (sin nombre)
fleet-certificates-expired-days-ago = caducó hace { $days } días
fleet-certificates-days-left = quedan { $days } días
fleet-certificates-renew = RENOVAR
fleet-certificates-not-certbot-acme-sh = no es certbot/acme.sh
fleet-certificates-nothing-read-yet-read-again = Todavía no se ha leído nada: LEER DE NUEVO lee todos los servidores accesibles.

## fleet-checks
fleet-checks-checks-from-outside = COMPROBACIONES DESDE FUERA
fleet-checks-checks-failing = { $count } comprobaciones · { $failing } fallando
fleet-checks-run-from-this-machine-only = se ejecutan desde este equipo, solo mientras Crow está abierto
fleet-checks-http-s = HTTP(S)
fleet-checks-tcp-port = PUERTO TCP
fleet-checks-dns-name = NOMBRE DNS
fleet-checks-every = CADA
fleet-checks-1-min = 1 min
fleet-checks-5-min = 5 min
fleet-checks-attach-to = VINCULAR A
fleet-checks-no-server = ningún servidor
fleet-checks-add-check = AÑADIR COMPROBACIÓN
fleet-checks-an-attached-checks-alerts-land = Las alertas de una comprobación vinculada van a ese servidor (su página, el mapa, el correo). Una comprobación avisa tras dos fallos seguidos y se resuelve con el primer éxito.
fleet-checks-on = {" "}· en { $s }
fleet-checks-cert-days = {" "}· cert. { $d } días
fleet-checks-checking = comprobando…
fleet-checks-not-checked-yet = aún sin comprobar
fleet-checks-alerts-on = ALERTAS ACTIVADAS
fleet-checks-alerts-off = ALERTAS DESACTIVADAS
fleet-checks-no-checks-yet-add-a = Todavía no hay comprobaciones: añade arriba una web, un puerto o un nombre DNS.
fleet-checks-every-s = { $target } · cada { $every_secs } s{ $server }

## fleet-drift
fleet-drift-not-yet = aún no
fleet-drift-just-now = ahora mismo
fleet-drift-min-ago = hace { $s } min
fleet-drift-h-ago = hace { $s } h
fleet-drift-search-every-servers-configs = BUSCAR EN LAS CONFIGURACIONES DE TODOS LOS SERVIDORES
fleet-drift-from-what-crow-has-read = a partir de lo que Crow ha leído; no se contacta con ningún servidor
fleet-drift-nothing-found-crow-searches-the = No se encontró nada. Crow busca en los archivos que ha leído: abre la pantalla Configuración de un servidor para leer sus archivos.
fleet-drift-first-matches = primeras { $n } coincidencias
fleet-drift-text = { $n } { $n ->
        [one] coincidencia
       *[other] coincidencias
    }
fleet-drift-the-file = el archivo
fleet-drift-drift-from-baselines = DESVIACIÓN DE LAS LÍNEAS BASE
fleet-drift-drifted-files-re-read-every = { $drifted } desviados · los archivos se releen cada 15 min mientras Crow funciona · comprobado { $ago }
fleet-drift-check-now = COMPROBAR AHORA
fleet-drift-reading = LEYENDO { $count }…
fleet-drift-no-baselines-pinned-yet-pin = Todavía no hay líneas base fijadas. Fija una desde la pantalla Configuración de un servidor (historial de un archivo → FIJAR COMO LÍNEA BASE); todos los servidores de ese grupo, o la flota, se comparan entonces con ella.
fleet-drift-drifted = { $n_drift } desviados
fleet-drift-all-match = todos coinciden
fleet-drift-bring-all-back = RESTAURAR LOS { $n_drift }
fleet-drift-matches-comments-spacing-differ = coincide (cambian comentarios/espacios)
fleet-drift-text-2 = { $count } { $count ->
        [one] diferencia
       *[other] diferencias
    }: { $cloned }
fleet-drift-vs-baseline = frente a la línea base de { $scope }
fleet-drift-what-changed = QUÉ CAMBIÓ
fleet-drift-baseline-vs-server = { $name } · { $path } · línea base (−) frente a servidor (+)
fleet-drift-crow-kept-only-hashes-of = Crow solo guardó el hash de una de las dos: el contenido de los archivos se sella con la clave de la bóveda, y no había ninguna cuando se leyó (o la bóveda está bloqueada ahora). Puede saber que difieren, pero no en qué, y no puede enviar la línea base ni fijar esta copia. Configura la bóveda (Ajustes → Bóveda y seguridad) y luego COMPROBAR AHORA.
fleet-drift-bring-back-writes-the-baseline = Restaurar escribe la línea base sobre la copia de este servidor (comprobada por su validador, guardada en el historial). Aceptar convierte esta copia en la línea base: todos los demás servidores de su ámbito se comparan entonces con ella.

## fleet-dr
fleet-dr-plans = PLANES
fleet-dr-new = + NUEVO
fleet-dr-no-plans-yet-a-plan = Todavía no hay planes. Un plan dice qué importa en un servidor, dónde van sus copias de seguridad, en cuánto tiempo debe volver y cómo reconstruirlo.
fleet-dr-drill-min = {" "}· simulacro { $minutes } min{ $d }
fleet-dr-name = NOMBRE
fleet-dr-rto-min = RTO (MIN)
fleet-dr-rpo-min = RPO (MIN)
fleet-dr-for = PARA
fleet-dr-group = grupo { $g }
fleet-dr-what-matters = QUÉ IMPORTA
fleet-dr-backups-path-or-glob-timer = COPIAS (RUTA O PATRÓN | TEMPORIZADOR)
fleet-dr-runbook-markdown-with-commands = PROCEDIMIENTO (MARKDOWN, CON COMANDOS)
fleet-dr-save = GUARDAR
fleet-dr-checking = COMPROBANDO…
fleet-dr-check-against-the-servers = COMPROBAR CONTRA LOS SERVIDORES

fleet-dr-export-md = EXPORTAR (.md)
fleet-dr-delete-plan = ELIMINAR PLAN
fleet-dr-drills = SIMULACROS
fleet-dr-rebuild-it-for-real-somewhere = Reconstrúyelo de verdad en algún sitio inofensivo (una VM de Multipass, un servidor de pruebas) siguiendo el procedimiento; Crow lo cronometra frente al RTO.
fleet-dr-running-for-min = en curso desde hace { $chrono } min
fleet-dr-finish-drill = TERMINAR SIMULACRO
fleet-dr-start-a-drill = EMPEZAR UN SIMULACRO
fleet-dr-met-the-rto = cumplió el RTO
fleet-dr-missed-the-rto = no cumplió el RTO
fleet-dr-min = { $when } · { $minutes } min · { $d }{ $d2 }

## fleet-group-modal
fleet-group-modal-server = Servidor
fleet-group-modal-group = GRUPO · { $server_name }
fleet-group-modal-move-this-server-to-a = Mover este servidor a un grupo:
fleet-group-modal-no-group = sin grupo

## fleet-hardening
fleet-hardening-not-yet = aún no
fleet-hardening-min-ago = hace { $chrono } min
fleet-hardening-posture = POSTURA
fleet-hardening-open-critical-accepted = { $open } abiertos ({ $crit } críticos) · { $accepted } aceptados
fleet-hardening-read-as-root-where-crow = leído { $checked }, como root donde Crow puede (sshd -T, el estado del cortafuegos, ss)
fleet-hardening-reading = LEYENDO…
fleet-hardening-check-now = COMPROBAR AHORA
fleet-hardening-export-report-md = EXPORTAR INFORME (.md)
fleet-hardening-reason-to-accept = MOTIVO PARA ACEPTAR
fleet-hardening-fix-everywhere = corregir en todos:
fleet-hardening-every-fix-first-proves-a = Cada corrección comprueba primero que funciona un inicio de sesión nuevo con clave, luego cambia una sola cosa, comprueba que se aplicó (sshd -t, sshd -T, el estado del cortafuegos) y demuestra que un inicio de sesión nuevo sigue funcionando. Lo que falle se deshace. Crow nunca pone PermitRootLogin no donde entra como root, y el cortafuegos siempre permite el puerto SSH.
fleet-hardening-crow-logs-in-as = Crow entra como { $user }
fleet-hardening-not-read-unreachable-or-not = sin leer (inaccesible, o aún sin comprobar)
fleet-hardening-nothing-found = no se encontró nada
fleet-hardening-text = { $count } { $count ->
        [one] hallazgo
       *[other] hallazgos
    }
fleet-hardening-accepted-by = aceptado por { $by }: { $reason }
fleet-hardening-fix = corregir: { $f }

## fleet-import
fleet-import-import-from-providers = IMPORTAR DESDE PROVEEDORES
fleet-import-asking-account-s = consultando { $loading } cuenta(s)…
fleet-import-instance-s-not-in-the = { $count } instancia(s) · { $importable } fuera de la flota
fleet-import-instances-already-in-the-fleet = Las instancias que ya están en la flota se emparejan por IP y se vinculan, así que su región viene del proveedor. IMPORTAR abre Añadir servidor ya rellenado: la clave de host y el acceso se comprueban como siempre.
fleet-import-linked-enrolled-server-s-to = { $linked } servidor(es) añadidos vinculados a sus instancias.
fleet-import-no-address = sin dirección
fleet-import-in-fleet-as = en la flota como { $server }
fleet-import-import = IMPORTAR →

## fleet-incidents
fleet-incidents-from-an-alert-last-7 = DESDE UNA ALERTA (ÚLTIMOS 7 DÍAS)
fleet-incidents-no-alerts-in-the-last = No hubo alertas en los últimos 7 días.
fleet-incidents-resolved = {" "}· resuelta
fleet-incidents-or-a-window-on-a = O UNA VENTANA EN UN SERVIDOR
fleet-incidents-pick-an-alert-or-a = Elige una alerta o una ventana: la cronología reúne alrededor las alertas, los cambios del propio Crow, las comprobaciones fallidas y los registros de error (err y peores).
fleet-incidents-reading-logs = {" "}· leyendo registros…
fleet-incidents-summarizing = RESUMIENDO…
fleet-incidents-summarize-clanker = RESUMIR (CLANKER)
fleet-incidents-export-md = EXPORTAR (.md)
fleet-incidents-via-a-likely-sequence-an = mediante { $via } · una secuencia probable, la lectura de una IA: compárala con la cronología
fleet-incidents-nothing-in-this-window = Nada en esta ventana.
fleet-incidents-events = { $when } → { $when2 } · { $count } eventos{ $st }

## fleet-lab-modal
fleet-lab-modal-local-vms-containers = VMS Y CONTENEDORES LOCALES
fleet-lab-modal-multipass-vms-and-lab-containers = VMs de Multipass y contenedores de laboratorio de este equipo, añadidos a Crow
fleet-lab-modal-multipass-vms = VMS DE MULTIPASS
fleet-lab-modal-ubuntu-vms-on-this-machine = VMs de Ubuntu en este equipo: lánzalas, impórtalas y enciéndelas
fleet-lab-modal-local-lab-containers = CONTENEDORES DE LABORATORIO LOCALES
fleet-lab-modal-lab-containers-on-this-machine = Contenedores de laboratorio de este equipo, añadidos a Crow como nodos de pruebas
fleet-lab-modal-checking = comprobando…
fleet-lab-modal-not-installed = no instalado
fleet-lab-modal-discovered-local-test-nodes = NODOS DE PRUEBAS LOCALES DETECTADOS
fleet-lab-modal-detected = { $count } detectados
fleet-lab-modal-refresh = ACTUALIZAR 🔄
fleet-lab-modal-no-local-test-nodes-found = No se encontraron nodos de pruebas locales
fleet-lab-modal-use-the-form-below-to = Usa el formulario de abajo para crear un nodo de pruebas nuevo de Ubuntu o Debian
fleet-lab-modal-close = CERRAR
fleet-lab-modal-endpoint = { $distro_display } · Punto de acceso: { $endpoint_display }
fleet-lab-modal-stop = ■ DETENER
fleet-lab-modal-start = ▶ INICIAR
fleet-lab-modal-enrolled = ✓ AÑADIDO
fleet-lab-modal-enroll-in-crow = + AÑADIR A CROW
fleet-lab-modal-checking-2 = COMPROBANDO…
fleet-lab-modal-not-installed-2 = NO INSTALADO
fleet-lab-modal-daemon-not-running = EL SERVICIO NO ESTÁ EN MARCHA
fleet-lab-modal-needs-a-fix = { $daemon } · NECESITA UN ARREGLO
fleet-lab-modal-ready = LISTO · { $daemon }
fleet-lab-modal-ubuntu-vms-on-this-machine-2 = VMs de Ubuntu en este equipo. Crow las lanza con su propia clave, fija sus claves de host a través de Multipass y las añade a la flota como servidores SSH.
fleet-lab-modal-no-vms-yet-launch-one = Todavía no hay VMs: lanza una abajo.
fleet-lab-modal-copy = COPIAR
fleet-lab-modal-command-copied = Comando copiado
fleet-lab-modal-in-fleet = EN LA FLOTA
fleet-lab-modal-import-to-crow = IMPORTAR A CROW
fleet-lab-modal-no-address = sin dirección
fleet-lab-modal-its-fleet-entry-goes-to = {" "}Su entrada en la flota pasa al archivo.
fleet-lab-modal-delete-for-good = ELIMINAR DEFINITIVAMENTE
fleet-lab-modal-cancel = CANCELAR
fleet-lab-modal-launch-a-vm = LANZAR UNA VM
fleet-lab-modal-cpu = CPU
fleet-lab-modal-memory = MEMORIA
fleet-lab-modal-gb = { $m } GB
fleet-lab-modal-disk = DISCO
fleet-lab-modal-gb-2 = { $d } GB
fleet-lab-modal-newest-lts = LTS más reciente
fleet-lab-modal-launch-add-to-fleet = LANZAR Y AÑADIR A LA FLOTA
fleet-lab-modal-delete-and-its-disk-for = ¿Eliminar { $name } y su disco definitivamente?{ $in_fleet }

## fleet-lab-state
fleet-lab-state-starting = Iniciando { $name }…
fleet-lab-state-stopping = Deteniendo { $name }…
fleet-lab-state-restarting = Reiniciando { $name }…
fleet-lab-state-suspending = Suspendiendo { $name }…
fleet-lab-state-deleting-and-its-disk = Eliminando { $name } y su disco…

## fleet-logsearch
fleet-logsearch-search = BUSCAR
fleet-logsearch-priority = PRIORIDAD
fleet-logsearch-when = CUÁNDO
fleet-logsearch-at-most = COMO MÁXIMO
fleet-logsearch-where = DÓNDE
fleet-logsearch-every-server = todos los servidores
fleet-logsearch-group = grupo { $g }
fleet-logsearch-search-2 = BUSCAR ⏎
fleet-logsearch-searching = BUSCANDO EN { $count }…
fleet-logsearch-save-as = GUARDAR COMO
fleet-logsearch-save = GUARDAR
fleet-logsearch-saved = GUARDADAS
fleet-logsearch-alert = ALERTA { $level }
fleet-logsearch-alert-off = ALERTA DESACTIVADA
fleet-logsearch-in = {" "}en { $unit }
fleet-logsearch-prio = {" "}· prio. ≤{ $p }
fleet-logsearch-group-2 = grupo { $group }

fleet-logsearch-an-alerting-search-runs-every = Una búsqueda con alerta se ejecuta cada 10 minutos sobre los últimos 15, en sus servidores, mientras Crow está abierto; la alerta se resuelve cuando deja de coincidir (y se envía por correo con el plugin Email).
fleet-logsearch-lines = { $count } LÍNEAS
fleet-logsearch-limit = {" "}(límite)
fleet-logsearch-summarizing = RESUMIENDO…
fleet-logsearch-summarize-clanker = RESUMIR (CLANKER)
fleet-logsearch-via-only-the-matching-lines = mediante { $via } · solo se enviaron las líneas que coinciden · la lectura de una IA: compruébala

## fleet-overview
fleet-overview-none-enrolled = ninguno añadido
fleet-overview-1-group = 1 grupo
fleet-overview-groups = { $n } grupos
fleet-overview-1-region = 1 región · { $group_note }
fleet-overview-regions = { $n } regiones · { $group_note }
fleet-overview-avg-1m-load-vcpu = carga media 1 min · { $total_vcpu } vCPU
fleet-overview-no-metrics-yet = aún sin métricas
fleet-overview-not-read-yet = aún sin leer
fleet-overview-past-the-rotation-policy = { $overdue } fuera de la política de rotación · { $note }
fleet-overview-not-read-yet-2 = { $note } · { $unknown } aún sin leer
fleet-overview-not-computed-yet = aún sin calcular
fleet-overview-no-baselines-pinned = ninguna línea base fijada
fleet-overview-match-baselines-as-last-read = { $count } { $count ->
        [one] archivo coincide
       *[other] archivos coinciden
    } con las líneas base · según la última lectura
fleet-overview-on-as-last-read = en { $count } { $count ->
        [one] servidor
       *[other] servidores
    } · según la última lectura
fleet-overview-servers = SERVIDORES
fleet-overview-open-alerts = ALERTAS ABIERTAS
fleet-overview-crit-warn = { $crit } críticas · { $warn } avisos
fleet-overview-config-drift = DESVIACIÓN DE CONFIG.
fleet-overview-fleet-load = CARGA DE LA FLOTA
fleet-overview-oldest-host-key = CLAVE DE HOST MÁS ANTIGUA
fleet-overview-connected = CONECTADOS
fleet-overview-reachable-over-their-transport = accesibles por su transporte
fleet-overview-launch-vm = + LANZAR VM
fleet-overview-local-lab = LABORATORIO LOCAL
fleet-overview-ok = OK
fleet-overview-checking = COMPROBANDO
fleet-overview-via = { $host } mediante { $name }
fleet-overview-no-servers-enrolled = Ningún servidor añadido
fleet-overview-all-reachable = Los { $connected_count } accesibles
fleet-overview-checking-2 = Comprobando { $n } { $n ->
        [one] servidor
       *[other] servidores
    }
fleet-overview-acknowledged = { $message } · reconocida
fleet-overview-crow-wasnt-running-from-to = Crow no estuvo en marcha desde { $fmt } hasta { $fmt2 }: en ese tiempo no se vigiló nada.
fleet-overview-all = TODOS
fleet-overview-production = PRODUCCIÓN
fleet-overview-staging = PREPRODUCCIÓN
fleet-overview-dev-lab = DESARROLLO / LAB
fleet-overview-by-region = POR REGIÓN ({ $regions })
fleet-overview-by-region-2 = POR REGIÓN
fleet-overview-by-group = POR GRUPO ({ $groups_count })
fleet-overview-by-group-2 = POR GRUPO
fleet-overview-hosts-sort-health-click-to = { $server_count } hosts · orden por estado ↓ · clic para abrir
fleet-overview-of-hosts-click-to-open = { $visible_count } de { $server_count } hosts · clic para abrir
fleet-overview-unknown = DESCONOCIDA
fleet-overview-detecting = DETECTANDO…
fleet-overview-detect-regions = DETECTAR REGIONES ↻
fleet-overview-group = GRUPO:
fleet-overview-all-2 = TODOS ({ $total_count })
fleet-overview-no-group = SIN GRUPO
fleet-overview-create = CREAR
fleet-overview-cancel = CANCELAR
fleet-overview-new-group = + GRUPO NUEVO
fleet-overview-host = HOST
fleet-overview-address = DIRECCIÓN
fleet-overview-os = SO
fleet-overview-role = ROL
fleet-overview-env = ENTORNO
fleet-overview-cpu = CPU
fleet-overview-mem = MEM
fleet-overview-disk = DISCO
fleet-overview-uptime = ACTIVO
fleet-overview-alerts = ALERTAS
fleet-overview-no-servers-enrolled-2 = NINGÚN SERVIDOR AÑADIDO
fleet-overview-connect-remote-linux-servers-via = Conecta servidores Linux remotos por SSH o crea nodos de laboratorio locales y aislados.
fleet-overview-add-remote-server-ssh = + AÑADIR SERVIDOR REMOTO (SSH)
fleet-overview-fleet-alerts = ALERTAS DE LA FLOTA
fleet-overview-open = { $count } abiertas
fleet-overview-ack-all = RECONOCER TODAS
fleet-overview-activity = ACTIVIDAD
fleet-overview-audit-log-all-hosts = registro de auditoría · todos los hosts →
fleet-overview-no-recent-fleet-activity-recorded = No hay actividad reciente registrada en la flota
fleet-overview-dismiss = DESCARTAR
fleet-overview-rolling-reboot = REINICIO ESCALONADO
fleet-overview-push-baseline-to-all = ENVIAR LÍNEA BASE A TODOS
fleet-overview-revoke-sessions = REVOCAR SESIONES
fleet-overview-rotate-host-keys = ROTAR CLAVES DE HOST
fleet-overview-fleet-wide = EN TODA LA FLOTA
fleet-overview-one-host-at-a-time = un host cada vez · se detiene al primer fallo
fleet-overview-rolling-reboot-2 = Reinicio escalonado
fleet-overview-reboots-every-reachable-server-in = Reinicia por turnos cada servidor accesible; cada uno vuelve de un arranque limpio antes de pasar al siguiente.
fleet-overview-push-baseline = Enviar línea base
fleet-overview-pushes-each-config-baseline-to = Envía cada línea base de configuración a los servidores cuya copia se ha desviado de ella.
fleet-overview-rotate-all-host-keys = Rotar todas las claves de host
fleet-overview-new-ssh-host-keys-on = Claves de host SSH nuevas en cada servidor, fijadas de nuevo en known_hosts y comprobadas con un inicio de sesión nuevo; se deshace ante cualquier fallo. Otros equipos verán una clave distinta.
fleet-overview-revoke-all-sessions = Revocar todas las sesiones
fleet-overview-ends-every-ssh-login-on = Cierra todas las sesiones SSH de todos los servidores excepto la del propio Crow. Las sesiones de consola y las claves no se tocan: quien tenga una clave que aún funcione puede volver a entrar.

## fleet-patching
fleet-patching-just-now = ahora mismo
fleet-patching-min-ago = hace { $s } min
fleet-patching-h-ago = hace { $s } h
fleet-patching-d-ago = hace { $s } d
fleet-patching-of-behind-on-security-updates = { $behind } de { $count } con actualizaciones de seguridad pendientes · { $reboots } esperando un reinicio
fleet-patching-every-server = todos los servidores
fleet-patching-group = grupo { $g }
fleet-patching-then-reboot-servers-that-need = después reiniciar los servidores que lo necesiten, los que están tras un bastión antes que el bastión, esperando a que vuelvan los servicios
fleet-patching-each-run-lists-the-exact = Cada ejecución lista los paquetes exactos por servidor y te pide que escribas una palabra antes de ejecutar nada. Un servidor cada vez; el primer fallo la detiene. Solo se actualizan los paquetes listados (apt --only-upgrade, dnf upgrade, apk add --upgrade). Los servidores fuera de la ventana de su grupo, los inaccesibles y el equipo donde corre Crow quedan fuera, con el motivo.
fleet-patching-whos-behind = QUIÉN VA ATRASADO
fleet-patching-read-every-6-h-while = leído { $ago } · cada 6 h mientras Crow funciona
fleet-patching-reading = LEYENDO { $count }…
fleet-patching-refresh = ACTUALIZAR
fleet-patching-server = SERVIDOR
fleet-patching-group-2 = GRUPO
fleet-patching-via = MEDIANTE
fleet-patching-security = SEGURIDAD
fleet-patching-all = TODAS
fleet-patching-kernel = KERNEL
fleet-patching-read = LEÍDO
fleet-patching-reading-2 = leyendo…
fleet-patching-not-read-yet = aún sin leer
fleet-patching-couldnt-read = no se pudo leer: { $error }
fleet-patching-needed = NECESARIO
fleet-patching-maintenance-windows = VENTANAS DE MANTENIMIENTO
fleet-patching-per-group-local-time-a = por grupo, hora local · una ejecución deja fuera los servidores fuera de la suya · en blanco: a cualquier hora
fleet-patching-no-groups-yet-windows-are = Todavía no hay grupos: las ventanas se definen por grupo (Flota → POR GRUPO).

fleet-patching-open-now = abierta ahora
fleet-patching-closed-now = cerrada ahora
fleet-patching-any-time = a cualquier hora
fleet-patching-examples-sun-02-05-mon = Ejemplos: Sun 02-05 · Mon-Fri 22-06 (pasada la medianoche cuenta como el día en que empieza) · Sat,Sun 01-04 · daily 03-05. Intro guarda.

## fleet-people
fleet-people-people = PERSONAS
fleet-people-accounts-keys-servers-read = { $count } cuentas · { $key_rows_count } claves · { $scans_count } servidores leídos
fleet-people-read-again = LEER DE NUEVO
fleet-people-reading = LEYENDO { $count }…
fleet-people-couldnt-read = no se pudo leer: { $failed }
fleet-people-who-can-log-in-where = QUIÉN PUEDE ENTRAR DÓNDE
fleet-people-text = { $count } { $count ->
        [one] servidor
       *[other] servidores
    }
fleet-people-offboard-everywhere-their-keys-are = Dar de baja en todas partes: primero se revocan sus claves, luego la cuenta se bloquea y caduca (reversible) o se elimina (se conserva la carpeta personal). Los servidores donde Crow entra con esta cuenta quedan fuera.
fleet-people-find-a-key-everywhere = BUSCAR UNA CLAVE EN TODAS PARTES
fleet-people-every-key-in-any-authorized = todas las claves de cualquier authorized_keys, con nombre cuando es una de Crow
fleet-people-crow = {" "}(Crow)
fleet-people-skips-crows-own-logins = (omite los accesos del propio Crow)
fleet-people-an-account-with-their-key = una cuenta con su clave en los servidores de un grupo (se crea donde falte)
fleet-people-every-server = todos los servidores
fleet-people-group = grupo { $g }
fleet-people-with-sudo = con sudo

## fleet-rollouts
fleet-rollouts-one-change-across-many-servers = Un cambio en muchos servidores: primero el canario (la ejecución se pausa para que lo compruebes), luego el resto en lotes que se ejecutan a la vez. El primer fallo lo detiene todo, con la salida de ese servidor.
fleet-rollouts-what = QUÉ
fleet-rollouts-a-command = UN COMANDO
fleet-rollouts-a-config-file = UN ARCHIVO DE CONFIGURACIÓN
fleet-rollouts-path-enter-reads-every-servers = RUTA (INTRO LEE LA COPIA DE CADA SERVIDOR)
fleet-rollouts-read = LEER
fleet-rollouts-from-a-baseline = desde una línea base:
fleet-rollouts-new-content = CONTENIDO NUEVO
fleet-rollouts-each-server-checks-it-with = Cada servidor lo comprueba antes con el validador propio del formato (sshd -t, nginx -t, visudo, fstab…); el contenido rechazado no se escribe. Cada escritura se guarda en el historial de configuración de ese servidor.
fleet-rollouts-where = DÓNDE
fleet-rollouts-every-server = todos los servidores
fleet-rollouts-group = grupo { $g }
fleet-rollouts-left-out-unreachable-or-the = {" "}· { $count } fuera (inaccesibles, o el equipo donde corre Crow)
fleet-rollouts-canary-runs-first-alone = CANARIO · VA PRIMERO, SOLO
fleet-rollouts-no-canary = sin canario
fleet-rollouts-then-at-a-time = LUEGO, DE CADA VEZ
fleet-rollouts-preview-the-run = VER LA EJECUCIÓN →
fleet-rollouts-a-command-has-no-preview = Un comando no tiene vista previa: pruébalo antes en el canario.
fleet-rollouts-what-changes-per-server = QUÉ CAMBIA, POR SERVIDOR
fleet-rollouts-type-the-path-and-press = Escribe la ruta y pulsa Intro para leer la copia de cada servidor.
fleet-rollouts-new-file = archivo nuevo
fleet-rollouts-reading = leyendo…
fleet-rollouts-identical-this-server-is-left = Idéntico: este servidor queda fuera.
fleet-rollouts-no-such-file-there-yet = Ese archivo todavía no existe ahí: se creará.
fleet-rollouts-couldnt-read-it = No se pudo leer: { $e }
fleet-rollouts-reading-2 = Leyendo…
fleet-rollouts-start-from-this-servers-copy = PARTIR DE LA COPIA DE ESTE SERVIDOR
fleet-rollouts-text = { $count } { $count ->
        [one] servidor
       *[other] servidores
    }{ $left_out }

## fleet-run-panel
fleet-run-panel-nothing-to-run = Nada que ejecutar.
fleet-run-panel-pausing-after-the-canary = , con pausa tras el canario
fleet-run-panel-one-host-at-a-time = { $count } { $count ->
        [one] paso
       *[other] pasos
    }, un host cada vez; el primer fallo detiene la ejecución.
fleet-run-panel-the-canary-is-done-check = El canario ha terminado. Compruébalo y luego CONTINUAR con el resto, o PARAR aquí.
fleet-run-panel-stopping-after-the-current-host = Parando después del host actual…
fleet-run-panel-running = Ejecutando…
fleet-run-panel-not-run = sin ejecutar
fleet-run-panel-left-out = fuera · { $why }
fleet-run-panel-type-to-start = ESCRIBE { $keyword } PARA EMPEZAR
fleet-run-panel-continue-with-the-rest = CONTINUAR CON EL RESTO
fleet-run-panel-stop-here = PARAR AQUÍ
fleet-run-panel-stop-after-this-host = PARAR TRAS ESTE HOST
fleet-run-panel-close = CERRAR
fleet-run-panel-cancel = CANCELAR
fleet-run-panel-in-the-first-failure-stops = { $count } { $count ->
        [one] paso
       *[other] pasos
    } en { $groups_count } { $groups_count ->
        [one] etapa
       *[other] etapas
    }{ $run }; el primer fallo detiene la ejecución.

## fleet-run
fleet-run-done = { $count } hechos
fleet-run-already-fine = { $skipped } ya estaban bien
fleet-run-stopped-at = detenido en { $failed }
fleet-run-stopped-on-request = detenido a petición
fleet-run-not-run = { $not_run } sin ejecutar

## fleet-setup
fleet-setup-default-unassigned = por defecto / sin asignar
fleet-setup-close-esc = CERRAR esc
fleet-setup-enrolled-servers = SERVIDORES AÑADIDOS:
fleet-setup-fleet-groups = GRUPOS DE LA FLOTA:
fleet-setup-unique-tags = ETIQUETAS ÚNICAS:
fleet-setup-host-keys-pinned = CLAVES DE HOST FIJADAS:
fleet-setup-none-enrolled = NINGUNO AÑADIDO
fleet-setup-fleet-topology-group-assignments = ASIGNACIÓN DE GRUPOS
fleet-setup-manage-new-group = + GESTIONAR / GRUPO NUEVO
fleet-setup-no-servers-currently-enrolled-in = Ahora mismo no hay servidores en la flota.
fleet-setup-text = { $member_count } { $member_count ->
        [one] SERVIDOR
       *[other] SERVIDORES
    }
fleet-setup-change-group = CAMBIAR GRUPO ▾
fleet-setup-host-keys = CLAVES DE HOST
fleet-setup-crow-pins-each-servers-host = Crow fija la huella de la clave de host de cada servidor al añadirlo y se niega a conectar si cambia. Las antigüedades salen de las fechas de los archivos de clave en cada servidor; todavía no hay política de rotación.
fleet-setup-fingerprint-pinned = Huella fijada
fleet-setup-not-pinned = Sin fijar
fleet-setup-keys-under-a-year-old = Claves de menos de un año
fleet-setup-keys-1-2-years-old = Claves de 1–2 años
fleet-setup-keys-over-2-years-old = Claves de más de 2 años
fleet-setup-key-age-not-read-yet = Antigüedad de la clave aún sin leer
fleet-setup-fleet-tag-taxonomy = TAXONOMÍA DE ETIQUETAS DE LA FLOTA
fleet-setup-no-custom-tags-defined-assign = No hay etiquetas propias definidas. Asigna etiquetas a los nodos al añadirlos o en la configuración del servidor para clasificar tu infraestructura.
fleet-setup-agentless-execution-guarantee = GARANTÍA DE EJECUCIÓN SIN AGENTE
fleet-setup-crow-talks-plain-ssh-and = Crow habla SSH sin más y no instala nada en tus servidores. Las acciones en toda la flota aún no existen; cuando existan, se ejecutarán en un host cada vez y se detendrán al primer fallo.
fleet-setup-text-2 = { $count } { $count ->
        [one] servidor
       *[other] servidores
    }
fleet-setup-groups-policies = Grupos y políticas
fleet-setup-patching = Parches
fleet-setup-rollouts = Despliegues
fleet-setup-drift-search = Desviación y búsqueda

fleet-setup-hardening = Endurecimiento
fleet-setup-certificates = Certificados
fleet-setup-people = Personas y claves
fleet-setup-log-search = Búsqueda en registros
fleet-setup-checks = Comprobaciones
fleet-setup-incidents = Incidentes
fleet-setup-dr-plans = PLANES DE RECUPERACIÓN
fleet-setup-team = Equipo
fleet-setup-audit-log = Auditoría
fleet-setup-fleet-map = MAPA DE LA FLOTA
fleet-setup-nav-change = CAMBIOS
fleet-setup-nav-security = SEGURIDAD
fleet-setup-nav-watch = VIGILANCIA
fleet-setup-nav-organisation = ORGANIZACIÓN
fleet-setup-nav-plugins = PLUGINS
fleet-setup-fleet-configuration = CONFIGURACIÓN DE LA FLOTA
fleet-setup-fleet-configuration-about = lo que cambia cómo funciona la flota y lo que les pasa a sus servidores
fleet-setup-topology-dr = TOPOLOGÍA Y DR
fleet-setup-topology-dr-about = la flota tal como está, y cómo se recupera tras un desastre
titlebar-topology-dr = Topología y DR
titlebar-fleet-configuration = Configuración de la flota
app-palette-dr-plans = Planes de recuperación

## fleet-state
fleet-state-no-data = SIN DATOS
fleet-state-connected-but-the-metrics-probe = conectado, pero la sonda de métricas falló
fleet-state-stopped = DETENIDO
fleet-state-lab-container-isnt-answering = el contenedor de laboratorio no responde
fleet-state-the-metrics-probe-failed-on = la sonda de métricas falló en este equipo

## fleet-team
fleet-team-team = EQUIPO
fleet-team-team-mode-on = { $count } { $count ->
        [one] miembro
       *[other] miembros
    } · modo equipo activado
fleet-team-off-crow-is-single-user = desactivado: Crow es de un solo usuario
fleet-team-sign-out-switch = CERRAR SESIÓN / CAMBIAR
fleet-team-who-uses-this-crow-with = Quién usa este Crow, con qué rol y en qué servidores. Cada miembro entra con su propia frase de paso; su rol se aplica en cada acción que cambia un servidor (acciones de servicios, escrituras de configuración, usuarios y cortafuegos, ejecuciones de flota), y cada registro de cambios lleva su nombre. Los miembros viven en la bóveda de este equipo: compartirlos entre equipos requiere cuentas de Crow (más adelante).
fleet-team-you = {" "}(tú)
fleet-team-fleet-runs-need-a-second = las ejecuciones de flota necesitan la aprobación de un segundo miembro (su frase de paso, escrita en la ejecución)
fleet-team-add-a-member = AÑADIR UN MIEMBRO
fleet-team-start-a-team-the-first = CREAR UN EQUIPO: EL PRIMER MIEMBRO ES ADMINISTRADOR (TÚ)
fleet-team-role = ROL
fleet-team-custom = PERSONALIZADO
fleet-team-can = PUEDE
fleet-team-on = EN
fleet-team-group = grupo { $g }
fleet-team-env = entorno { $e }
fleet-team-none-picked-the-whole-fleet = (ninguno elegido: toda la flota; los entornos mandan sobre los grupos)
fleet-team-add-member = AÑADIR MIEMBRO
fleet-team-start-the-team = CREAR EL EQUIPO
fleet-team-whos-at-the-keyboard = ¿Quién está al teclado?
fleet-team-this-crow-has-a-team = Este Crow tiene un equipo: inicia sesión para que se aplique tu rol y tus cambios lleven tu nombre.
fleet-team-sign-in = INICIAR SESIÓN

## history
history-at = a las { $fmt }
history-cpu = CPU
history-memory = MEMORIA
history-load-1m = CARGA (1 min)
history-history = HISTORIAL
history-one-sample-a-minute-while = una muestra por minuto mientras Crow está abierto · { $readout }
history-now = { $chrono } → ahora
history-crow-wasnt-running-nothing-watched = Crow no estaba en marcha: nada vigilado
history-server-unreachable = servidor inaccesible
history-alert-open-warn = alerta abierta (aviso)
history-alert-open-crit = alerta abierta (crítica)

## lock
lock-crow-vault-locked = BÓVEDA DE CROW BLOQUEADA
lock-password-2fa = CONTRASEÑA + 2FA
lock-store-config-crow-crow-db = ALMACÉN: ~/.config/crow/crow.db
lock-2fa-mandatory = 2FA: OBLIGATORIO
lock-master-password = CONTRASEÑA MAESTRA
lock-hide = OCULTAR
lock-show = MOSTRAR
lock-2fa-totp-code = CÓDIGO 2FA TOTP
lock-6-digits = 6 DÍGITOS
lock-tab-to-switch-to-unlock = Tab para cambiar · ↵ para desbloquear
lock-unlock-vault = DESBLOQUEAR BÓVEDA

## lock-setup
lock-setup-security-notice-enabling-password-logon = AVISO DE SEGURIDAD · ACTIVAR EL ACCESO CON CONTRASEÑA
lock-setup-mandatory-2fa = 2FA OBLIGATORIO
lock-setup-you-are-about-to-activate = Vas a activar la protección cifrada local de la bóveda de Crow. Antes de seguir, revisa estas reglas de seguridad esenciales:
lock-setup-1-password-means-password-and = 1. CONTRASEÑA SIGNIFICA CONTRASEÑA Y 2FA
lock-setup-there-is-no-password-only = No existe el acceso solo con contraseña. Crow exige autenticación de dos factores (TOTP RFC 6238) en cada desbloqueo. En el siguiente paso emparejarás una app de autenticación (1Password, Google Authenticator, YubiKey).
lock-setup-2-what-the-password-protects = 2. QUÉ PROTEGE LA CONTRASEÑA
lock-setup-crow-encrypts-its-secrets-provider = Crow cifra sus secretos (tokens de proveedores, claves de API de IA) con ChaCha20-Poly1305. Con contraseña, su clave queda bloqueada por ella (Argon2id, 64 MB, 3 iteraciones) en lugar de por el llavero del sistema, nunca se escribe en disco sin cifrar y se borra de la RAM al bloquear. El resto de ~/.config/crow/crow.db (lista de servidores, ajustes, historial de cambios) no está cifrado. Tus claves SSH se quedan donde están, en ~/.ssh.
lock-setup-3-no-cloud-backup-or = 3. SIN COPIA EN LA NUBE NI RECUPERACIÓN DE CONTRASEÑA
lock-setup-this-is-a-local-only = Es un sistema solo local, de un usuario y sin telemetría. Si pierdes la contraseña maestra o tu autenticador 2FA, los secretos que guardó Crow (tokens de proveedores, claves de IA) no se pueden recuperar y hay que volver a introducirlos. Tu lista de servidores y tus ajustes siguen siendo legibles.
lock-setup-cancel = CANCELAR
lock-setup-i-understand-setup-password-2fa = LO ENTIENDO · CONFIGURAR CONTRASEÑA Y 2FA →
lock-setup-configure-password-mandatory-2fa = CONFIGURAR CONTRASEÑA Y 2FA OBLIGATORIO
lock-setup-step-2-of-2 = PASO 2 DE 2
lock-setup-master-password = CONTRASEÑA MAESTRA
lock-setup-hide-password = OCULTAR CONTRASEÑA
lock-setup-show-password = MOSTRAR CONTRASEÑA
lock-setup-confirm-master-password = CONFIRMAR CONTRASEÑA MAESTRA
lock-setup-mandatory-two-factor-authentication-totp = AUTENTICACIÓN DE DOS FACTORES OBLIGATORIA (TOTP)
lock-setup-required = OBLIGATORIO
lock-setup-scan-the-code-with-your = Escanea el código con tu app de autenticación (1Password, Google Authenticator, Aegis…) o introduce esta clave:
lock-setup-copied = ✓ COPIADA
lock-setup-copy-key = COPIAR CLAVE
lock-setup-enter-6-digit-code-to = INTRODUCE EL CÓDIGO DE 6 DÍGITOS PARA CONFIRMAR EL EMPAREJAMIENTO:
lock-setup-back = ← ATRÁS
lock-setup-activate-password-2fa = ACTIVAR CONTRASEÑA Y 2FA

## logs
logs-systemd-journal-logs = REGISTROS DEL JOURNAL DE SYSTEMD
logs-entries = { $filtered_count }/{ $total_count } entradas
logs-volatile-lost-on-reboot = ⚠ VOLÁTIL — SE PIERDE AL REINICIAR
logs-no-age-limit = sin límite de antigüedad
logs-no-size-cap = sin límite de tamaño
logs-persistent = { $days } · { $size } PERSISTENTE
logs-explain = ✦ EXPLICAR
logs-boundaries = ⚙ LÍMITES: { $bound_text }
logs-live-tail-on = EN DIRECTO
logs-paused = EN PAUSA
logs-export = EXPORTAR ⧉
logs-show-all = MOSTRAR TODO
logs-clear = LIMPIAR
logs-range = PERIODO
logs-boot = ARRANQUE
logs-dedupe-repeats = AGRUPAR REPETIDOS
logs-quick = RÁPIDO
logs-load-older = ↑ cargar { $limit } anteriores
logs-all = TODOS
logs-err = ERR+
logs-warn = AVISO+
logs-info = INFO+
logs-kernel = KERNEL
logs-crow-actions = ✓ ACCIONES DE CROW
logs-crow-actions-2 = ACCIONES DE CROW
logs-pid = PID { $fpid }
logs-terminate-pid = ¿terminar el PID { $fpid }?
logs-kill-sigterm = MATAR (SIGTERM) ⏎
logs-cancel = cancelar
logs-kill-pid = ⏻ MATAR PID
logs-timestamp = FECHA Y HORA
logs-level = NIVEL
logs-unit-identifier = UNIDAD / IDENTIFICADOR
logs-pid-2 = PID
logs-message = MENSAJE
logs-no-journal-log-entries-matched = Ninguna entrada del journal coincide con tu filtro
logs-try-clearing-the-search-query = Prueba a borrar la búsqueda o a elegir el nivel 'TODOS'
logs-structured-systemd-journal-fields = CAMPOS ESTRUCTURADOS DEL JOURNAL DE SYSTEMD
logs-copy-message = COPIAR MENSAJE
logs-copy-json = COPIAR JSON
logs-asking-about-lines = Preguntando a { $provider } sobre { $lines_sent } líneas…
logs-lines-sent = { $provider } · { $lines_sent } líneas enviadas
logs-get-a-plain-english-reading = Obtén una lectura en lenguaje llano de las líneas en pantalla: qué está pasando, qué parece ir mal, las causas probables y qué comprobar después.

logs-explain-logs = ✦ EXPLICAR REGISTROS
logs-working = trabajando…
logs-no-provider = ningún proveedor
logs-sends-the-log-lines-on = Envía a { $p } las líneas de registro en pantalla (no los nombres de host). No se envía nada hasta que hagas clic.
logs-add-an-api-key-for = Añade una clave de API para un proveedor en Ajustes → Clankers para activarlo.

## logs-retention-modal
logs-retention-modal-systemd-journal-retention-boundaries = RETENCIÓN Y LÍMITES DEL JOURNAL DE SYSTEMD
logs-retention-modal-eliminate-30-minute-silent-log = Acaba con las rotaciones silenciosas de 30 minutos y evita borrados por caídas
logs-retention-modal-critical-storage-is-volatile-auto = ⚠ CRÍTICO: EL ALMACENAMIENTO ES VOLÁTIL / AUTO
logs-retention-modal-if-this-host-crashes-or = Si este host se cae o se apaga, los registros se borran para siempre de la RAM.
logs-retention-modal-lock-to-disk = FIJAR EN DISCO 🔒
logs-retention-modal-crash-safe-persistent-storage-active = ALMACENAMIENTO PERSISTENTE A PRUEBA DE CAÍDAS ACTIVO
logs-retention-modal-active-journals-persist-in-var = Los journals activos persisten en /var/log/journal/ ante caídas y reinicios del servidor.
logs-retention-modal-current-disk-usage = USO DE DISCO ACTUAL
logs-retention-modal-active-files = { $active_files_count } archivos activos
logs-retention-modal-retention-horizon = HORIZONTE DE RETENCIÓN
logs-retention-modal-hours = { $retention_horizon_hours } horas
logs-retention-modal-oldest = Más antiguo: { $oldest_timestamp }
logs-retention-modal-daily-write-rate = ESCRITURA DIARIA
logs-retention-modal-mb-day = { $daily_burn_rate_mb } MB/día
logs-retention-modal-days-at-current-rate = ~{ $estimated_retained_days } días al ritmo actual
logs-retention-modal-allocation = ASIGNACIÓN: { $disk_usage_display } / { $max_use_display } ({ $usage_pct }%)
logs-retention-modal-systemmaxuse-boundary = Límite SystemMaxUse
logs-retention-modal-storage-persistence-storage = PERSISTENCIA DEL ALMACENAMIENTO (Storage)
logs-retention-modal-controls-whether-logs-survive-host = Controla si los registros sobreviven a reinicios y cortes de luz del host
logs-retention-modal-persistent-rec = PERSISTENTE (Rec.)
logs-retention-modal-volatile-ram-only = VOLÁTIL (solo RAM)
logs-retention-modal-auto-fragile = AUTO (frágil)
logs-retention-modal-max-disk-quota-systemmaxuse = CUOTA MÁXIMA DE DISCO (SystemMaxUse)
logs-retention-modal-hard-ceiling-before-older-journals = Techo fijo antes de que roten los journals antiguos
logs-retention-modal-500-mb = 500 MB
logs-retention-modal-2-0-gb = 2,0 GB
logs-retention-modal-4-0-gb-rec = 4,0 GB (Rec.)
logs-retention-modal-8-0-gb = 8,0 GB
logs-retention-modal-16-gb = 16 GB
logs-retention-modal-custom = Personalizado
logs-retention-modal-time-horizon-maxretentionsec = HORIZONTE TEMPORAL (MaxRetentionSec)
logs-retention-modal-maximum-age-before-logs-are = Antigüedad máxima antes de borrar los registros
logs-retention-modal-7-days = 7 días
logs-retention-modal-14-days = 14 días
logs-retention-modal-30-days-rec = 30 días (Rec.)
logs-retention-modal-90-days = 90 días
logs-retention-modal-180-days = 180 días
logs-retention-modal-unlimited = Ilimitado
logs-retention-modal-safety-disk-reserve-systemkeepfree = RESERVA DE DISCO DE SEGURIDAD (SystemKeepFree)
logs-retention-modal-guarantees-disk-space-stays-free = Garantiza espacio libre en disco para evitar caídas por disco raíz lleno
logs-retention-modal-1-0-gb = 1,0 GB
logs-retention-modal-2-0-gb-rec = 2,0 GB (Rec.)
logs-retention-modal-5-0-gb = 5,0 GB
logs-retention-modal-edit-in-crow-config = EDITAR EN CROW-CONFIG ↗
logs-retention-modal-apply-to-etc-systemd-journald = APLICAR A /etc/systemd/journald.conf

## logs-state
logs-state-oom-kills = MUERTES POR OOM
logs-state-segfaults = SEGFAULTS
logs-state-firewall-blocks = BLOQUEOS DEL CORTAFUEGOS
logs-state-disk-errors = ERRORES DE DISCO
logs-state-ssh-login-failures = FALLOS DE ACCESO SSH

## onboard
onboard-connect = Conectar
onboard-verify = Verificar
onboard-name = Nombre
onboard-check-connection = COMPROBAR CONEXIÓN →
onboard-next-name-it = SIGUIENTE: PONLE NOMBRE →
onboard-add-server = AÑADIR SERVIDOR ⏎
onboard-add-server-2 = AÑADIR SERVIDOR
onboard-cancel-esc = CANCELAR esc
onboard-back = ← ATRÁS
onboard-where-is-it-and-how = ¿Dónde está y cómo entra Crow?
onboard-crow-connects-over-ssh-and = Crow se conecta por SSH y no instala nada. El siguiente paso comprueba la clave de host del servidor antes de enviar nada.
onboard-or-a-lab-container-on = O UN CONTENEDOR DE LABORATORIO EN ESTE EQUIPO
onboard-address = DIRECCIÓN
onboard-port = PUERTO
onboard-user = USUARIO
onboard-reach-it = CÓMO LLEGAR
onboard-directly = Directamente
onboard-via = mediante { $route }
onboard-only-reachable-through-another-server = ¿Solo se llega a través de otro servidor? Marca antes ese como bastión: el interruptor «bastión» de su cabecera.
onboard-every-connection-goes-through-the = Todas las conexiones pasan por el bastión, entrando allí con la propia clave del bastión. Cómo llega el bastión a este servidor depende de su red; Crow solo necesita la dirección tal como la ve el bastión.
onboard-pick-a-bastion-when-this = Elige un bastión cuando a este servidor solo se llegue a través de uno.
onboard-log-in-with = ENTRAR CON
onboard-a-key-in-crow = Una clave de Crow
onboard-a-key-from-settings-keys = Una clave de Ajustes → Claves, ya presente en el servidor.
onboard-a-password-once = Una contraseña, una vez
onboard-crow-logs-in-once-installs = Crow entra una vez, instala su propia clave y olvida la contraseña.
onboard-my-ssh-agent = Mi agente SSH
onboard-whatever-your-agent-and-ssh = Lo que usarían tu agente y ~/.ssh/config.
onboard-no-keys-in-crow-yet = Todavía no hay claves en Crow: usa «Una contraseña, una vez» o añade una en Ajustes → Claves y rotación.
onboard-used-for-one-login-only = Se usa para un único inicio de sesión, tras comprobar la clave de host. Nunca se guarda.
onboard-crow-wont-know-which-key = Crow no sabrá qué clave es, así que no podrá rotarla ni auditarla.
onboard-checking = Comprobando…
onboard-not-checked-yet = Aún sin comprobar.
onboard-no-answer = sin respuesta
onboard-logging-in-and-reading-the = Entrando y leyendo el servidor…
onboard-reached-host-key-trusted-logged = Accesible, clave de host de confianza, sesión iniciada.
onboard-the-host-key-is-trusted = La clave de host es de confianza, pero no se pudo iniciar sesión: mira el registro de abajo.
onboard-the-host-key-changed-since = La clave de host cambió desde la última vez que se confió en ella.
onboard-reached-check-the-host-key = Accesible. Revisa abajo la clave de host y luego confía en ella.
onboard-through = a través de { $v }
onboard-checking-2 = COMPROBANDO…
onboard-check-again = COMPROBAR DE NUEVO
onboard-host-key-trusted = CLAVE DE HOST · DE CONFIANZA
onboard-host-key-changed-not-trusted = CLAVE DE HOST · CAMBIADA, SIN CONFIANZA
onboard-host-key-new-to-this = CLAVE DE HOST · NUEVA PARA ESTE EQUIPO
onboard-compare-it-with-the-servers = Compárala con la del propio servidor, p. ej. desde su consola: ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub

onboard-it-matches-trust-it = ✓ COINCIDE: CONFIAR
onboard-possibly-a-reinstalled-server-possibly = Puede ser un servidor reinstalado, o alguien en medio. Si el cambio es esperado, quita tú la entrada antigua de ~/.ssh/known_hosts y vuelve a comprobar.
onboard-log = REGISTRO
onboard-distro = DISTRIBUCIÓN
onboard-kernel = KERNEL
onboard-arch = ARQ.
onboard-memory = MEMORIA
onboard-disk = DISCO
onboard-init = INIT
onboard-firewall = CORTAFUEGOS
onboard-open-ports = PUERTOS ABIERTOS
onboard-time-sync = SINCRONIZACIÓN HORARIA
onboard-what-crow-read = LO QUE LEYÓ CROW
onboard-config-tools = HERRAMIENTAS DE CONFIG.
onboard-crow-knows-debian-and-red = Crow conoce la disposición de configuración de las familias Debian y Red Hat; en esta distribución puede que el descubrimiento de configuración no encuentre algunos archivos.
onboard-your-ssh-agent = tu agente SSH
onboard-password-once = contraseña, una vez
onboard-how-should-it-show-in = ¿Cómo debe aparecer en la flota?
onboard-all-of-this-can-be = Todo esto se puede cambiar después.
onboard-name-2 = NOMBRE
onboard-environment = ENTORNO
onboard-group = GRUPO
onboard-tags = ETIQUETAS
onboard-bastion = Bastión
onboard-other-servers-can-be-added = Se pueden añadir otros servidores a través de este.
onboard-through-2 = a través de { $v } ·{" "}
onboard-group-2 = {" "}· grupo { $group }
onboard-crow-couldnt-log-in-yet = Crow aún no pudo entrar: el servidor se añadirá, pero aparecerá como inaccesible hasta que pueda.
onboard-couldnt-reach-it = No se pudo llegar a él: { $r }
onboard-logs-in-with = { $via }entra con { $login }{ $state }{ $state2 }

## onboard-probe
onboard-probe-is-not-a-valid-host = { $host } no es un host válido
onboard-probe-could-not-resolve = no se pudo resolver { $host }
onboard-probe-tcp = tcp { $host }:{ $port }: { $e }
onboard-probe-tcp-connect-failed = la conexión tcp a { $host }:{ $port } falló: { $e }
onboard-probe-tcp-connect = conexión tcp a { $host }:{ $port }
onboard-probe-ssh-banner = banner ssh { $banner }
onboard-probe-no-ssh-banner-is-this = sin banner SSH — ¿es un puerto SSH?
onboard-probe-host-key = clave de host { $host_key_fingerprint }
onboard-probe-could-not-fetch-the-servers = no se pudieron obtener las claves de host del servidor (ssh-keyscan)
onboard-probe-couldnt-make-a-scratch-known = no se pudo crear un known_hosts temporal: { $e }
onboard-probe-no-answer = sin respuesta
onboard-probe-let-crow-in-but-couldnt = { $via } dejó entrar a Crow pero no pudo llegar a { $host }:{ $port }, o { $via } rechazó a Crow
onboard-probe-through = a través de { $via }: { $why }
onboard-probe-reached-through = se llegó a { $host }:{ $port } a través de { $via }
onboard-probe-ssh-banner-2 = banner ssh { $b }
onboard-probe-through-2 = a través de { $via }
onboard-probe-host-key-matches-ssh-known = la clave de host coincide con ~/.ssh/known_hosts
onboard-probe-host-key-changed-known-hosts = CLAVE DE HOST CAMBIADA: known_hosts tiene otra clave para este host — posible ataque de intermediario; no se confía en ella
onboard-probe-host-key-not-in-known = la clave de host no está en known_hosts — verifica la huella y luego acéptala
onboard-probe-no-home-directory = no hay carpeta personal
onboard-probe-could-not-read-facts-over = no se pudieron leer los datos por SSH: { $e }
onboard-probe-privileges = privilegios: { $v }
onboard-probe-unknown-location = ubicación desconocida
onboard-probe-region = región { $place } · { $provider } { $code }
onboard-probe-cloud-metadata = metadatos de nube
onboard-probe-ufw-installed = ufw instalado
onboard-probe-read-os-release-uname-meminfo = leídos os-release, uname, meminfo, df y los puertos en escucha

## overview-collector
overview-collector-unit-on = { $action } la unidad { $unit } en { $label }
overview-collector-sent-signal-to-pid-on = Señal { $signal } enviada al PID { $pid } en { $label }

## overview-dashboard
overview-dashboard-open = ABRIR →
overview-dashboard-services = SERVICIOS
overview-dashboard-active = ACTIVOS
overview-dashboard-failed = CON FALLOS
overview-dashboard-degraded = DEGRADADOS
overview-dashboard-inactive = INACTIVOS
overview-dashboard-needs-attention = REQUIERE ATENCIÓN
overview-dashboard-no-failed-or-degraded-units = No hay unidades con fallos ni degradadas.
overview-dashboard-cpu = { $cpu }% CPU
overview-dashboard-processes = PROCESOS
overview-dashboard-total = TOTAL
overview-dashboard-zombie = ZOMBI
overview-dashboard-blocked-d = BLOQUEADOS (D)
overview-dashboard-top-cpu = MÁS CPU
overview-dashboard-no-processes-sampled-yet = Todavía no hay procesos muestreados.
overview-dashboard-top-memory = MÁS MEMORIA
overview-dashboard-sockets = SOCKETS
overview-dashboard-listening = ESCUCHANDO
overview-dashboard-all-interfaces = TODAS LAS INTERFACES
overview-dashboard-interface = INTERFAZ
overview-dashboard-local-only = SOLO LOCAL
overview-dashboard-listening-ports-widest-exposure-first = PUERTOS EN ESCUCHA · LOS MÁS EXPUESTOS PRIMERO
overview-dashboard-no-listening-sockets-sampled-yet = Todavía no hay sockets en escucha muestreados.
overview-dashboard-updates-security = ACTUALIZACIONES Y SEGURIDAD
overview-dashboard-checked = comprobado { $checked_label }
overview-dashboard-checking = COMPROBANDO…
overview-dashboard-refresh = ACTUALIZAR ↻
overview-dashboard-reading-the-package-manager-and = Leyendo el gestor de paquetes y buscando CVEs…
overview-dashboard-not-checked-yet = Aún sin comprobar.
overview-dashboard-pending = PENDIENTES
overview-dashboard-security = SEGURIDAD
overview-dashboard-cves-fixed-by-updating = CVES CORREGIDOS AL ACTUALIZAR
overview-dashboard-apply-security = APLICAR { $security } DE SEGURIDAD
overview-dashboard-apply-all = APLICAR LAS { $count }
overview-dashboard-kernel = KERNEL
overview-dashboard-running = en uso { $u }
overview-dashboard-update-pending = actualización { $kind ->
        [security] de seguridad
       *[kernel] del kernel
    } pendiente: { $change }
overview-dashboard-reboot-required-to-finish-installed = hace falta reiniciar para terminar de instalar las actualizaciones
overview-dashboard-package-lists-refreshed-h-ago = listas de paquetes actualizadas hace { $age_h } h
overview-dashboard-package-lists-are-days-old = las listas de paquetes tienen { $h } días — ejecuta `apt update` para resultados actuales
overview-dashboard-cves-fixed-by-pending-updates = CVES CORREGIDOS POR ACTUALIZACIONES PENDIENTES
overview-dashboard-none-found-for-the-pending = No se encontró ninguno para las actualizaciones pendientes.
overview-dashboard-more = +{ $fixed } más
overview-dashboard-known-cves-in-running-software = { $no_fix_yet } CVEs conocidos en el software en uso aún no tienen corrección
overview-dashboard-cves = CVES
overview-dashboard-pending-updates-security-first = ACTUALIZACIONES PENDIENTES · SEGURIDAD PRIMERO
overview-dashboard-everything-is-up-to-date = Todo está al día.
overview-dashboard-more-2 = +{ $pending } más
overview-dashboard-cve-data-osv-dev-crow = Datos de CVE: OSV.dev · Crow solo envía nombres y versiones de paquetes, nunca nombres de host

## overview-log-tail
overview-log-tail-log-tail = REGISTRO EN DIRECTO
overview-log-tail-expand = AMPLIAR ↗
overview-log-tail-error = ERROR { $err_count }
overview-log-tail-warn = AVISO { $warn_count }
overview-log-tail-info = INFO { $info_count }
overview-log-tail-stream-active = flujo activo
overview-log-tail-this-server = este servidor
overview-log-tail-socket-log-tail = REGISTRO DEL SOCKET
overview-log-tail-pid = PID { $p }
overview-log-tail-all-host-network-sockets-click = Todos los sockets de red del host · haz clic en una fila para seguirlo
overview-log-tail-filter-socket = FILTRO: SOCKET ✓

overview-log-tail-filter-all = FILTRO: TODO
overview-log-tail-time = HORA
overview-log-tail-level = NIVEL
overview-log-tail-unit = UNIDAD
overview-log-tail-pid-2 = PID
overview-log-tail-message = MENSAJE
overview-log-tail-lines-live-stream = { $count } líneas · en directo

## overview-models
overview-models-sleep = DORMIDO
overview-models-disk = DISCO
overview-models-zombie = ZOMBI
overview-models-idle = INACTIVO
overview-models-loopback = LOOPBACK
overview-models-private-net = RED PRIVADA
overview-models-public-internet = INTERNET PÚBLICO

## overview-service-inspector
overview-service-inspector-service-manager = GESTOR DE SERVICIOS
overview-service-inspector-select-a-service-on-the = Selecciona un servicio a la izquierda para gestionarlo
overview-service-inspector-pid = PID
overview-service-inspector-cpu = CPU
overview-service-inspector-mem = MEM
overview-service-inspector-rss = RSS
overview-service-inspector-uptime = ACTIVO
overview-service-inspector-did-not-reach-the-expected = { $unit } no llegó al estado esperado tras la última acción.
overview-service-inspector-retry = REINTENTAR
overview-service-inspector-lifecycle = CICLO DE VIDA
overview-service-inspector-view-logs-for-this-unit = VER LOS REGISTROS DE ESTA UNIDAD
overview-service-inspector-configuration = CONFIGURACIÓN
overview-service-inspector-edit = EDITAR { $file }
overview-service-inspector-open-in-crow-config-structured = abrir en crow-config — edición estructurada y validada
overview-service-inspector-recent-actions = ACCIONES RECIENTES
overview-service-inspector-ok = OK
overview-service-inspector-failed = FALLÓ
overview-service-inspector-pending = PENDIENTE
overview-service-inspector-start = ▶ INICIAR
overview-service-inspector-reload = ⟳ RECARGAR
overview-service-inspector-restart = ↻ REINICIAR
overview-service-inspector-stop = ■ DETENER
overview-service-inspector-no-active-connections-will-be = No se cortará ninguna conexión activa.
overview-service-inspector-established-and-listening-will-be = Se cortarán { $established } { $established ->
        [one] conexión establecida
       *[other] conexiones establecidas
    } y { $listening } { $listening ->
        [one] socket en escucha
       *[other] sockets en escucha
    }.
overview-service-inspector-checking-active-connections = Comprobando las conexiones activas…
overview-service-inspector-cancel = Cancelar
overview-service-inspector-asking-the-clanker-what-this = Preguntando al clanker qué es este servicio…
overview-service-inspector-low-risk = RIESGO BAJO
overview-service-inspector-medium-risk = RIESGO MEDIO
overview-service-inspector-high-risk = RIESGO ALTO
overview-service-inspector-critical = CRÍTICO
overview-service-inspector-to-stop-or-restart = al detener o reiniciar: { $reason }
overview-service-inspector-via-the-backup-the-primary = mediante el respaldo, { $via } · el principal falló: { $why }
overview-service-inspector-via-an-ais-reading-check = mediante { $via } · la lectura de una IA: compruébala antes de actuar

## overview-services-table
overview-services-table-table = TABLA
overview-services-table-map = MAPA
overview-services-table-log-drawer-active = PANEL DE REGISTROS (ACTIVO)
overview-services-table-log-drawer = PANEL DE REGISTROS
overview-services-table-sort-port = orden puerto ↑
overview-services-table-sort-status-name = orden estado · nombre
overview-services-table-sort-cpu = orden cpu ↓
overview-services-table-no-processes-match-try-all = Ningún proceso coincide — prueba con TODOS o borra la búsqueda
overview-services-table-no-services-match-try-all = Ningún servicio coincide — prueba con TODOS o borra la búsqueda
overview-services-table-of = { $first }–{ $last } de { $total }
overview-services-table-prev = ‹ ANT.
overview-services-table-next = SIG. ›
overview-services-table-group-by-status = AGRUPAR POR ESTADO
overview-services-table-group-by-name = AGRUPAR POR NOMBRE
overview-services-table-processes = { $total } procesos
overview-services-table-running = { $running } en marcha
overview-services-table-high-cpu = { $high_cpu } con mucha cpu
overview-services-table-sockets = { $total } sockets
overview-services-table-listening = { $listen } escuchando
overview-services-table-established = { $estab } establecidos
overview-services-table-units = { $total } unidades
overview-services-table-active = { $active } activas
overview-services-table-degraded = { $degraded } degradadas
overview-services-table-failed = { $failed } con fallos
overview-services-table-pid = PID
overview-services-table-user = USUARIO
overview-services-table-cpu = CPU %
overview-services-table-mem = MEM %
overview-services-table-rss = RSS
overview-services-table-stat = ESTADO
overview-services-table-time = TIEMPO
overview-services-table-command = COMANDO
overview-services-table-actions = ACCIONES
overview-services-table-proto = PROTO
overview-services-table-local-endpoint-firewall = EXTREMO LOCAL / CORTAFUEGOS
overview-services-table-peer-endpoint = EXTREMO REMOTO
overview-services-table-state = ESTADO
overview-services-table-process-service = PROCESO / SERVICIO
overview-services-table-unit = UNIDAD
overview-services-table-uptime = ACTIVO
overview-services-table-no-systemd-service-units-detected = No se detectaron unidades de servicio de systemd en el host activo
overview-services-table-no-active-connections-will-be = No se cortará ninguna conexión activa.
overview-services-table-established-and-listening-will-be = Se cortarán { $established } { $established ->
        [one] conexión establecida
       *[other] conexiones establecidas
    } y { $listening } { $listening ->
        [one] socket en escucha
       *[other] sockets en escucha
    }.
overview-services-table-checking-active-connections = Comprobando las conexiones activas…
overview-services-table-restart = ¿Reiniciar { $name }? { $blast_radius_text }
overview-services-table-cancel = Cancelar{" "}
overview-services-table-esc = esc
overview-services-table-restart-2 = REINICIAR ⏎
overview-services-table-no-processes-detected-on-active = No se detectaron procesos en el host activo
overview-services-table-terminate-pid-send-sigterm = ¿Terminar { $comm } (PID { $pid })? Se envía SIGTERM.
overview-services-table-kill-sigterm = MATAR (SIGTERM) ⏎
overview-services-table-no-open-network-sockets-detected = No se detectaron sockets de red abiertos en el host activo
overview-services-table-allow = permitir n.º { $rule_number }
overview-services-table-deny = denegar n.º { $rule_number }
overview-services-table-default-allow = permitido por defecto
overview-services-table-no-rule = sin regla
overview-services-table-fw-off = cf. inactivo
overview-services-table-fw = cf. ?

## overview-sockets-map
overview-sockets-map-gb = { $bytes } GB
overview-sockets-map-mb = { $bytes } MB
overview-sockets-map-kb = { $bytes } KB
overview-sockets-map-incoming-peers = EXTREMOS ENTRANTES
overview-sockets-map-remote-clients-initiating-traffic-into = Clientes remotos que inician tráfico hacia servicios en escucha
overview-sockets-map-local-processes-daemons = PROCESOS Y SERVICIOS LOCALES
overview-sockets-map-host-processes-maintaining-open-sockets = Procesos del host con sockets abiertos
overview-sockets-map-outgoing-peers = EXTREMOS SALIENTES
overview-sockets-map-external-endpoints-reached-by-host = Extremos externos a los que llegan los procesos del host
overview-sockets-map-hide-127-0-0-1 = OCULTAR 127.0.0.1

overview-sockets-map-focus = FOCO: { $p_name }
overview-sockets-map-showing-of-established-connections = MOSTRANDO { $shown_conns } DE { $total_conns } CONEXIONES ESTABLECIDAS
overview-sockets-map-no-connections-in-this-category = No hay conexiones en esta categoría.
overview-sockets-map-ports = { $count } { $count ->
        [one] conexión
       *[other] conexiones
    } · puertos: { $ports_summary }
overview-sockets-map-no-active-process-connections = No hay conexiones de procesos activas.
overview-sockets-map-pid = PID { $p }
overview-sockets-map-proc-unprivileged = proc. sin privilegios
overview-sockets-map-in = ← { $in_count } entrantes
overview-sockets-map-out = → { $out_count } salientes
overview-sockets-map-protocol-state-direction = Protocolo: { $protocol } · Estado: { $state } · Dirección: { $direction }
overview-sockets-map-process-pid = Proceso: { $process } (PID { $pid })
overview-sockets-map-view-in-processes = VER EN PROCESOS →
overview-sockets-map-view-in-firewall = VER EN CORTAFUEGOS →

## overview-state
overview-state-all-traffic = TODO EL TRÁFICO
overview-state-incoming = ENTRANTE
overview-state-outgoing = SALIENTE
overview-state-live = ACTIVOS
overview-state-failed = CON FALLOS
overview-state-inactive = INACTIVOS
overview-state-all = TODOS
overview-state-apps = APPS
overview-state-root = ROOT
overview-state-cpu-1 = CPU > 1%
overview-state-kernel = KERNEL

## overview-summary
overview-summary-all-interfaces = TODAS LAS INTERFACES
overview-summary-interface = INTERFAZ
overview-summary-local = LOCAL
overview-summary-failed = CON FALLOS
overview-summary-degraded = DEGRADADOS

## settings-clankers
settings-clankers-clankers-llm-providers-keys = CLANKERS · PROVEEDORES DE LLM Y CLAVES
settings-clankers-configured = { $configured_count }/{ $count } CONFIGURADOS
settings-clankers-30d-cumulative-calls = Llamadas en 30 días:
settings-clankers-calls = { $total_calls_30d } llamadas
settings-clankers-primary-backup = Principal · respaldo:
settings-clankers-no-backup = sin respaldo
settings-clankers-ai-usability-engine-log-clarification = Motor de ayuda con IA (aclaración de registros y explicaciones sencillas)
settings-clankers-configure-api-keys-for-llm = Configura claves de API para proveedores de LLM. Crow solo los llama cuando lo pides, para explicar líneas del journal en lenguaje llano. Las claves se guardan cifradas en la bóveda de Crow (su clave la guarda el llavero del sistema, o la bloquea tu contraseña de bóveda si la tienes activada) y se envían por la entrada estándar de curl, nunca por la línea de comandos. Las líneas de registro que envías van a ese proveedor.
settings-clankers-keys-cant-be-saved-right = Ahora mismo no se pueden guardar claves: { $why }.
settings-clankers-supported-clanker-providers-30-day = PROVEEDORES DE CLANKERS ADMITIDOS Y TELEMETRÍA DE 30 DÍAS
settings-clankers-set-ends = configurada · termina en …{ $tail }
settings-clankers-no-key-configured = SIN CLAVE CONFIGURADA
settings-clankers-primary = ★ PRINCIPAL
settings-clankers-backup = ↺ RESPALDO
settings-clankers-ready = LISTO
settings-clankers-not-setup = SIN CONFIGURAR
settings-clankers-api-key = Clave de API:
settings-clankers-model = Modelo:
settings-clankers-endpoint = Punto de acceso:
settings-clankers-30-day-activity = Actividad en 30 días:
settings-clankers-calls-2 = { $calls_30d } llamadas
settings-clankers-last-used = Último uso: { $ts }
settings-clankers-never-used = Nunca usado
settings-clankers-edit-key = ⚙ Editar clave
settings-clankers-testing = probando…
settings-clankers-test-key = ⚡ Probar clave
settings-clankers-reset = Reiniciar
settings-clankers-remove-backup = Quitar respaldo
settings-clankers-make-backup = Hacer respaldo
settings-clankers-make-primary = Hacer principal
settings-clankers-ai-usability-sandbox-wtf-is = PRUEBAS CON IA: «¿QUÉ NARICES INTENTA DECIR ESTE REGISTRO?»
settings-clankers-eli5-assistant-preview = VISTA PREVIA DEL ASISTENTE «EXPLÍCAMELO FÁCIL»
settings-clankers-load-presets = Cargar ejemplos:
settings-clankers-oom-killer = OOM Killer
settings-clankers-segfault-in-libc = Segfault en libc
settings-clankers-systemd-failure = Fallo de systemd
settings-clankers-sample-raw-journal-kernel-panic = Línea de ejemplo del journal / pánico del kernel:
settings-clankers-asking = PREGUNTANDO…
settings-clankers-translate-log-eli5 = EXPLICAR REGISTRO ↵
settings-clankers-asking-the-provider = preguntando al proveedor…
settings-clankers-models-offered-for-this-key = { $count } modelos disponibles para esta clave
settings-clankers-load-models = CARGAR MODELOS
settings-clankers-configure-clanker-api-key = Configurar la clave de API del clanker { $p_name }
settings-clankers-api-key-stored-encrypted-in = Clave de API (se guarda cifrada en la bóveda de Crow; no se vuelve a mostrar):
settings-clankers-custom-api-endpoint-proxy-optional = Punto de acceso / proxy propio (opcional):
settings-clankers-save-config = GUARDAR CONFIG. ↵

## settings-config-table
settings-config-table-yes = SÍ
settings-config-table-no = NO
settings-config-table-ms = { $n } ms
settings-config-table-empty = (vacío)
settings-config-table-none = (ninguno)
settings-config-table-enable-setting = Activar el ajuste
settings-config-table-disable-setting = Desactivar el ajuste
settings-config-table-aggressive-timeout = Tiempo de espera agresivo
settings-config-table-standard-recommended = Estándar recomendado
settings-config-table-tolerant-of-high-latency-links = Tolera enlaces de alta latencia
settings-config-table-slow-satellite-dialup = Satélite lento / módem
settings-config-table-aggressive-ping = Ping agresivo
settings-config-table-relaxed-heartbeat = Latido relajado
settings-config-table-off = DESACTIVADO
settings-config-table-disable-keepalive-ping = Desactivar el ping keepalive
settings-config-table-2-attempts = 2 intentos
settings-config-table-fast-dead-peer-detection = Detección rápida de extremos caídos
settings-config-table-3-attempts = 3 intentos
settings-config-table-5-attempts = 5 intentos
settings-config-table-tolerate-transient-packet-drop = Tolera pérdidas de paquetes puntuales
settings-config-table-10-attempts = 10 intentos
settings-config-table-very-tolerant = Muy tolerante
settings-config-table-1s-high-resolution = 1 s (alta resolución)
settings-config-table-higher-local-remote-cpu-use = Más uso de CPU local/remoto
settings-config-table-2s-default = 2 s (por defecto)
settings-config-table-smooth-streaming-low-load = Fluido y con poca carga
settings-config-table-5s-battery-eco = 5 s (batería/eco)
settings-config-table-low-background-overhead = Poca carga en segundo plano
settings-config-table-10s-minimal = 10 s (mínimo)
settings-config-table-minimal-polling-footprint = Consultas mínimas
settings-config-table-chacha20-poly1305-fast-modern = chacha20-poly1305 (rápido y moderno)
settings-config-table-hardware-agnostic-aead = AEAD independiente del hardware
settings-config-table-aes128-gcm-aes-ni-accelerated = aes128-gcm (acelerado con AES-NI)
settings-config-table-fast-on-modern-intel-amd = Rápido en Intel/AMD modernos
settings-config-table-aes256-gcm-maximum-depth = aes256-gcm (máxima profundidad)
settings-config-table-high-security-environments = Entornos de alta seguridad
settings-config-table-aes256-gcm-only = Solo AES256-GCM
settings-config-table-default-ed25519-key = Clave Ed25519 por defecto

settings-config-table-dedicated-bastion-key = Clave dedicada para el bastión
settings-config-table-legacy-rsa-identity = Identidad RSA antigua
settings-config-table-risk = Riesgo: { $risk }
settings-config-table-preparing = preparando…
settings-config-table-fleet-page-background = FONDO DE LA PÁGINA FLOTA
settings-config-table-a-picture-behind-the-server = Una imagen detrás de la lista de servidores, atenuada y suavizada para que la tabla se siga leyendo bien. PNG, JPEG o WebP; Crow guarda su propia copia.
settings-config-table-change-picture = CAMBIAR IMAGEN…
settings-config-table-choose-picture = ELEGIR IMAGEN…
settings-config-table-remove = QUITAR
settings-config-table-general = GENERAL
settings-config-table-general-application-behavior = {"["}general] · comportamiento de la aplicación
settings-config-table-connection-ssh = CONEXIÓN Y SSH
settings-config-table-connection-applies-to-every-host = {"["}connection] · se aplica a todos los hosts salvo que se anule
settings-config-table-keys-rotation = CLAVES Y ROTACIÓN
settings-config-table-keys-key-distribution-policies = {"["}keys] · distribución de claves y políticas
settings-config-table-vault-security = BÓVEDA Y SEGURIDAD
settings-config-table-vault-encrypted-secrets-who-can = {"["}vault] · secretos cifrados y quién puede abrir Crow
settings-config-table-servers-archives = SERVIDORES Y ARCHIVO
settings-config-table-servers-archiving-and-how-long = {"["}servers] · archivado y cuánto tiempo se guardan los datos
settings-config-table-ui-components-lab = LABORATORIO DE COMPONENTES
settings-config-table-lab-gpui-component-testbed-sandbox = {"["}lab] · banco de pruebas de gpui-component
settings-config-table-clankers-ai-usability = CLANKERS (AYUDA CON IA)
settings-config-table-clankers-api-keys-log-eli5 = {"["}clankers] · claves de API y explicaciones de registros
settings-config-table-providers = PROVEEDORES
settings-config-table-providers-cloud-accounts-your-servers = {"["}providers] · cuentas de nube donde corren tus servidores
settings-config-table-plugins = PLUGINS
settings-config-table-plugins-integrations-off-until-you = {"["}plugins] · integraciones, desactivadas hasta que las actives
settings-config-table-personalisation = PERSONALIZACIÓN
settings-config-table-appearance-make-crow-yours-saved = {"["}appearance] · haz tuyo Crow · se guarda al cambiarlo
settings-config-table-password-logon-is-not-configured = EL ACCESO CON CONTRASEÑA NO ESTÁ CONFIGURADO
settings-config-table-disabled-default = DESACTIVADO (POR DEFECTO)
settings-config-table-without-a-password-anyone-using = Sin contraseña, cualquiera que use tu cuenta del equipo puede abrir Crow. Los secretos (tokens de proveedores, claves de IA) siguen cifrados, con la clave guardada en el llavero del sistema. Una contraseña maestra con autenticación de dos factores (TOTP RFC 6238) hace que Crow la pida antes de abrirse, y bloquea con ella la clave de los secretos.
settings-config-table-set-password-2fa = PONER CONTRASEÑA Y 2FA →
settings-config-table-password-2fa-vault-configured = BÓVEDA CON CONTRASEÑA + 2FA CONFIGURADA
settings-config-table-locked-stance = POSTURA BLOQUEADA
settings-config-table-database-and-keys-are-secured = La base de datos y las claves están protegidas. Desbloquear requiere la contraseña y un código TOTP de 6 dígitos. El bloqueo automático se activa tras el tiempo configurado.
settings-config-table-lock-vault-now-l = BLOQUEAR BÓVEDA AHORA (⇧⌘L)
settings-config-table-its = está
settings-config-table-theyre = están
settings-config-table-changed = CAMBIADO
settings-config-table-or-enter-custom-value = O ESCRIBE UN VALOR PROPIO:
settings-config-table-apply = APLICAR
settings-config-table-reset-section = RESTABLECER SECCIÓN
settings-config-table-edit-config-toml = EDITAR config.toml ⌘/
settings-config-table-save-s-pending = GUARDAR ⌘S ({ $total_changed } pendientes)
settings-config-table-save-s = GUARDAR ⌘S
settings-config-table-more-in-this-section-not = { $hidden_settings } { $hidden_settings ->
        [one] ajuste más de esta sección aún no está conectado
       *[other] ajustes más de esta sección aún no están conectados
    } a nada, así que { $hidden_settings2 } { $hidden_settings ->
        [one] oculto
       *[other] ocultos
    } en lugar de fingir que funcionan.

## settings-diff-rail
settings-diff-rail-in-sync = sincronizado
settings-diff-rail-pending-diff = CAMBIOS PENDIENTES
settings-diff-rail-configuration-in-sync = Configuración sincronizada
settings-diff-rail-working-copy-matches-config-crow = La copia de trabajo coincide con ~/.config/crow/config.toml
settings-diff-rail-affects-live-sessions = AFECTA A SESIONES ACTIVAS
settings-diff-rail-pending-ssh-connection-changes-take = Los cambios pendientes de conexión SSH se aplican al reconectar. Las sesiones activas conservan sus parámetros actuales.
settings-diff-rail-keychain = LLAVERO
settings-diff-rail-no-keys-enrolled-yet = Todavía no hay claves añadidas.

## settings-email-form
settings-email-form-just-now = ahora mismo
settings-email-form-min-ago = hace { $secs } min
settings-email-form-h-ago = hace { $secs } h
settings-email-form-d-ago = hace { $secs } d
settings-email-form-waiting-retried-every-5-min = {" "}· { $count } en espera, se reintenta cada 5 min
settings-email-form-in-the-next-digest = {" "}· { $count } en el próximo resumen
settings-email-form-in-the-next-digest-2 = { $count } en el próximo resumen
settings-email-form-security = SEGURIDAD
settings-email-form-smtp-server = SERVIDOR SMTP
settings-email-form-port = PUERTO
settings-email-form-no-encryption-the-password-and = Sin cifrado: la contraseña y el correo cruzan la red en claro. Solo para un relé en este equipo o en tu propia red.
settings-email-form-username = USUARIO
settings-email-form-password = CONTRASEÑA
settings-email-form-from = DE
settings-email-form-to-comma-separated = PARA (separados por comas)
settings-email-form-critical-alerts = ALERTAS CRÍTICAS
settings-email-form-warnings = AVISOS
settings-email-form-digest-every-minutes = RESUMEN CADA (MINUTOS)
settings-email-form-quiet-hours-digests-wait = HORAS DE SILENCIO (LOS RESÚMENES ESPERAN)
settings-email-form-resolved-alerts-send-nothing-alerts = Las alertas resueltas no envían nada. Las alertas ya abiertas cuando lo actives no se envían. Crow no tiene agente: el correo solo sale mientras Crow está en marcha.
settings-email-form-save = GUARDAR
settings-email-form-sending = ENVIANDO…
settings-email-form-send-test = ENVIAR PRUEBA
settings-email-form-test-failed = La prueba falló: { $m }
settings-email-form-last-send-failed = El último envío falló: { $e }{ $d }
settings-email-form-last-mail-sent = Último correo enviado { $ago }{ $d }

## settings-keys
settings-keys-scan-idle = Análisis en reposo
settings-keys-all = TODAS ({ $count })
settings-keys-group = + GRUPO
settings-keys-scan = ANALIZAR ⟳
settings-keys-path = + RUTA
settings-keys-generate-key = + GENERAR CLAVE
settings-keys-auto-monitored-paths = rutas vigiladas automáticamente
settings-keys-unenrolled-ssh-discovered-on-disk = { $count } { $count ->
        [one] CLAVE SSH SIN AÑADIR ENCONTRADA
       *[other] CLAVES SSH SIN AÑADIR ENCONTRADAS
    } EN DISCO
settings-keys-found-in-scan-paths-click = encontradas en las rutas analizadas · pulsa Importar para añadirlas
settings-keys-fp = huella: { $fingerprint }
settings-keys-import-to = + IMPORTAR A { $target_grp }
settings-keys-enrolled-keys-in-memory = CLAVES AÑADIDAS EN MEMORIA ({ $count })
settings-keys-filtered-by-group = filtrado por grupo: { $g }
settings-keys-all-groups = todos los grupos

settings-keys-persisted-in-config-crow-crow = guardado en ~/.config/crow/crow.db
settings-keys-no-enrolled-keys-matching-this = NINGUNA CLAVE AÑADIDA COINCIDE CON ESTE FILTRO
settings-keys-scan-ssh-to-import-existing = Analiza ~/.ssh para importar claves existentes, o pulsa «+ GENERAR CLAVE» para crear una.
settings-keys-private-key-linked = ● CLAVE PRIVADA VINCULADA
settings-keys-public-key-only = ○ SOLO CLAVE PÚBLICA
settings-keys-reachable-servers = SERVIDORES ACCESIBLES:
settings-keys-attach-host = + VINCULAR HOST
settings-keys-sha256-fingerprint = HUELLA SHA256:
settings-keys-path-2 = ruta: { $path }
settings-keys-comment = comentario: { $c }
settings-keys-copied-fingerprint-to-clipboard = Huella copiada al portapapeles
settings-keys-copy-fingerprint = COPIAR HUELLA
settings-keys-copied-openssh-public-key-to = Clave pública OpenSSH copiada al portapapeles
settings-keys-copy-public-key = COPIAR CLAVE PÚBLICA
settings-keys-enrolled = añadida: { $key }
settings-keys-edit-attach-servers = EDITAR / VINCULAR SERVIDORES ✎
settings-keys-switch-attached-servers-to-it = CAMBIAR A ELLA LOS SERVIDORES VINCULADOS
settings-keys-rotate = ROTAR
settings-keys-remove-from-memory = QUITAR DE LA MEMORIA ✕
settings-keys-scan-paths = RUTAS DE ANÁLISIS
settings-keys-add-path = + AÑADIR RUTA
settings-keys-default = por defecto
settings-keys-group-taxonomy = TAXONOMÍA DE GRUPOS
settings-keys-new-group = + GRUPO NUEVO
settings-keys-text = { $count } { $count ->
        [one] clave
       *[other] claves
    }
settings-keys-key-rotation-audit = ROTACIÓN Y AUDITORÍA DE CLAVES
settings-keys-private-keys-stay-on-this = Las claves privadas se quedan en este equipo; a los servidores solo van las públicas. ROTAR crea una clave nueva, la instala en cada servidor que usa la antigua, comprueba que entra y luego quita la antigua, un servidor cada vez. No hay política de rotación automática.
settings-keys-keypair-generated-and-enrolled = ✓ PAR DE CLAVES GENERADO Y AÑADIDO
settings-keys-generate-new-openssh-keypair = + GENERAR UN PAR DE CLAVES OPENSSH NUEVO
settings-keys-esc = esc
settings-keys-private-key-written-to-disk =
    Clave privada guardada en disco con permisos 0600:
    { $priv_path }
settings-keys-fingerprint = Huella: { $fp }
settings-keys-copied-public-key-to-clipboard = Clave pública copiada al portapapeles
settings-keys-key-name = NOMBRE DE LA CLAVE:
settings-keys-algorithm = ALGORITMO:
settings-keys-ed25519-recommended = Ed25519 (recomendado)
settings-keys-rsa-4096-bit = RSA de 4096 bits
settings-keys-comment-optional = COMENTARIO (OPCIONAL):
settings-keys-save-directory = CARPETA DE DESTINO:
settings-keys-tab-to-switch-fields-enter = tab para cambiar de campo · intro para generar
settings-keys-cancel = CANCELAR
settings-keys-generate-keypair = GENERAR PAR DE CLAVES ↵
settings-keys-add-ssh-key-scan-directory = AÑADIR CARPETA DE ANÁLISIS DE CLAVES SSH
settings-keys-enter-directory-path-e-g = Ruta de la carpeta (p. ej. ~/work-keys o /etc/ssh):
settings-keys-add-scan = AÑADIR Y ANALIZAR ↵
settings-keys-create-new-key-group = CREAR UN GRUPO DE CLAVES NUEVO
settings-keys-enter-group-name-e-g = Nombre del grupo (p. ej. Flota de pruebas, Bastiones de borde):
settings-keys-create-group = CREAR GRUPO ↵
settings-keys-edit-key-attached-servers = EDITAR CLAVE Y SERVIDORES VINCULADOS
settings-keys-name = NOMBRE:
settings-keys-group-assignment = GRUPO ASIGNADO:
settings-keys-attach-to-fleet-servers = VINCULAR A SERVIDORES DE LA FLOTA:
settings-keys-save-changes = GUARDAR CAMBIOS ↵

## settings-keys-state
settings-keys-state-scanned-found-new = { $count } { $count ->
        [one] ruta analizada
       *[other] rutas analizadas
    } · { $discovered_count } { $discovered_count ->
        [one] clave encontrada
       *[other] claves encontradas
    } ({ $new_count } nuevas)

## settings-lab
settings-lab-type-something-here-to-test = Escribe algo aquí para probar gpui-component...
settings-lab-type-to-reveal-clear-button = Escribe para que aparezca el botón de borrar...
settings-lab-search-fleet-by-tag-or = buscar en la flota por etiqueta o región...
settings-lab-enter-sensitive-secret = Introduce un secreto sensible...
settings-lab-fleet-deployment-manifest-env-production =
    # Manifiesto de despliegue de la flota
    env: production
    replicas: 4
    region: us-east-1
    auto_drain: true
settings-lab-ui-components-lab = LABORATORIO DE COMPONENTES
settings-lab-gpui-component-transition-testbed = · banco de pruebas de la transición a gpui-component
settings-lab-isolated-sandbox = ENTORNO AISLADO
settings-lab-evaluation-transition-sandbox = ENTORNO DE EVALUACIÓN Y TRANSICIÓN
settings-lab-this-page-runs-gpui-component = Esta página ejecuta widgets de gpui-component dentro de Crow para probar la selección de texto nativa, el parpadeo del cursor, los atajos del portapapeles, deshacer/rehacer y el moldeado de texto con HarfBuzz antes de migrar las pantallas existentes.
settings-lab-1-primary-native-gpui-input = 1. ENTRADA GPUI NATIVA PRINCIPAL
settings-lab-native-gpui-component-input = ENTRADA NATIVA DE GPUI-COMPONENT
settings-lab-rope-buffer-shaping = Búfer rope + moldeado
settings-lab-native-mouse-drag-selection-cmd =
    ✓ Selección nativa arrastrando el ratón
    ✓ Cmd+A, Cmd+C, Cmd+V, Cmd+Z (¡deshacer!)
    ✓ Saltos de palabra (Alt+flechas) y de fin de línea (Cmd+flechas)
settings-lab-2-input-variants-capabilities = 2. VARIANTES Y CAPACIDADES DE ENTRADA
settings-lab-password-input-with-native-mask = ENTRADA DE CONTRASEÑA (CON MÁSCARA NATIVA Y BOTÓN DE BORRAR):
settings-lab-search-filter-input-cleanable = ENTRADA DE BÚSQUEDA / FILTRO (CON BORRADO):
settings-lab-prefix-suffix-adornments = ADORNOS DE PREFIJO Y SUFIJO:
settings-lab-port-22 = PUERTO 22
settings-lab-3-6-digit-otp-authenticator = 3. ENTRADA DE CÓDIGO OTP DE 6 DÍGITOS
settings-lab-grouped-otp-cells-split-3 = CELDAS OTP AGRUPADAS (3-3 CON AVANCE Y RETROCESO AUTOMÁTICOS):
settings-lab-direct-replacement-candidate-for-vault = Candidato directo para sustituir las pantallas de bloqueo y configuración de la bóveda.
settings-lab-4-multi-line-textarea = 4. ÁREA DE TEXTO MULTILÍNEA
settings-lab-multi-line-editor-with-word = EDITOR MULTILÍNEA CON AJUSTE DE LÍNEA Y SELECCIÓN MULTILÍNEA:
settings-lab-5-gpui-component-button-matrix = 5. MATRIZ DE BOTONES DE GPUI-COMPONENT
settings-lab-primary-action = Acción principal
settings-lab-secondary = Secundario
settings-lab-ghost = Fantasma
settings-lab-destructive = Destructivo

## settings
settings-general = General
settings-connection-ssh = Conexión y SSH
settings-keys-rotation = Claves y rotación
settings-vault-security = Bóveda y seguridad
settings-servers-archives = Servidores y archivo
settings-ui-components-lab = Laboratorio de componentes
settings-clankers-ai = Clankers (IA)
settings-providers = Proveedores
settings-plugins = Plugins
settings-personalisation = Personalización
settings-in-sync = { $path_str } · sincronizado
settings-pending = { $path_str } · { $total_changed } { $total_changed ->
        [one] cambio pendiente
       *[other] cambios pendientes
    }
settings-no-servers = sin servidores
settings-1-server = 1 servidor
settings-servers = { $n } servidores
settings-settings = AJUSTES
settings-close-esc = CERRAR esc

## settings-plugins
settings-plugins-local-virtual-machines = MÁQUINAS VIRTUALES LOCALES
settings-plugins-containers = CONTENEDORES
settings-plugins-cloud-providers = PROVEEDORES DE NUBE
settings-plugins-notifications = NOTIFICACIONES
settings-plugins-plugins-what-crow-connects-to = PLUGINS · CON QUÉ SE CONECTA CROW

settings-plugins-on = { $on }/{ $count } ACTIVADOS
settings-plugins-re-check = VOLVER A COMPROBAR
settings-plugins-crow-does-nothing-with-these = Crow no hace nada con ellos hasta que los activas. Un plugin activado añade lo que trae donde corresponde (Multipass añade + LANZAR VM a la Flota; un proveedor de nube añade importar, encender y apagar e instantáneas); uno desactivado ni se sondea ni se muestra.
settings-plugins-checking = comprobando…
settings-plugins-not-installed-on-this-machine = no está instalado en este equipo
settings-plugins-not-in-this-build = no está en esta compilación
settings-plugins-not-checked-yet = aún sin comprobar
settings-plugins-no-account-yet-set-one = Todavía no hay cuenta · configura una en Proveedores →
settings-plugins-1-account-providers = 1 cuenta · Proveedores →
settings-plugins-accounts-providers = { $n } cuentas · Proveedores →
settings-user-plugins-installed = PLUGINS INSTALADOS
settings-user-plugins-open-folder = ABRIR CARPETA
settings-user-plugins-reading = LEYENDO…
settings-user-plugins-reload = RECARGAR
settings-user-plugins-about = Plugins escritos por ti o por otros, una carpeta cada uno. Crow revisa cada archivo y te muestra lo que un plugin puede ejecutar antes de que lo aceptes; nada se ejecuta hasta entonces. Si cambia un archivo, el plugin se desactiva hasta que lo revises de nuevo.
settings-user-plugins-reading-folder = Leyendo la carpeta de plugins…
settings-user-plugins-none = Aún no hay plugins. Pon la carpeta de un plugin en la carpeta de plugins y pulsa RECARGAR.
settings-user-plugins-by = de { $author }
settings-user-plugins-close = CERRAR
settings-user-plugins-why = POR QUÉ
settings-user-plugins-review = REVISAR
settings-user-plugins-secret-from-vault = secreto, de la bóveda
settings-user-plugins-effect-read = LEE
settings-user-plugins-effect-change = CAMBIA
settings-user-plugins-effect-danger = PELIGRO
settings-user-plugins-on-each-server-root = en cada servidor, como root
settings-user-plugins-on-each-server = en cada servidor
settings-user-plugins-on-this-machine = en esta máquina
settings-user-plugins-dry-run-first = primero un ensayo: { $action }
settings-user-plugins-typed-keyword = palabra a escribir: { $keyword }
settings-user-plugins-login-check = Crow comprueba su acceso antes y después
settings-user-plugins-input = pide { $key }
settings-user-plugins-needs-local = necesita en esta máquina
settings-user-plugins-needs-server = en los servidores
settings-user-plugins-needs-os = SO
settings-user-plugins-setup = INSTALACIÓN · UNA VEZ, EN ESTA MÁQUINA
settings-user-plugins-actions = ACCIONES
settings-user-plugins-settings = AJUSTES
settings-user-plugins-setting-secret = { $label } (secreto)
settings-user-plugins-page-server = página «{ $label }» en cada servidor
settings-user-plugins-page-fleet = página «{ $label }» en Configuración de la flota
settings-user-plugins-badge = una insignia, cada { $every }
settings-user-plugins-shows = MUESTRA
settings-user-plugins-file-binary = { $path } (binario)
settings-user-plugins-meta = para Crow { $crow } · { $files } archivos · { $license }
settings-user-plugins-changed-since = Un archivo cambió desde que aceptaste este plugin, así que está desactivado. Revisa lo que hay ahora.
settings-user-plugins-warnings = AVISOS
settings-user-plugins-refused-heading = RECHAZADO · CROW NO CARGARÁ ESTE PLUGIN
settings-user-plugins-files = ARCHIVOS
settings-user-plugins-confirm-accept = CONFIRMAR: ACEPTAR
settings-user-plugins-accept = ACEPTAR
settings-user-plugins-refuse = RECHAZAR
settings-user-plugins-revoke = RETIRAR
settings-user-plugins-confirm-explain = Aceptarlo permite que este plugin ejecute los comandos de arriba en tus servidores, como root donde lo indica, cuando tú o una ejecución en la flota los inicie. Pulsa CONFIRMAR: ACEPTAR para aceptarlo.
settings-user-plugins-install = INSTALAR
settings-user-plugins-reinstall = REINSTALAR
settings-user-plugins-installing = INSTALANDO…
settings-user-plugins-setup-heading = INSTALACIÓN EN ESTA MÁQUINA
settings-user-plugins-installed-for-these = Instalado para estos archivos.
settings-user-plugins-not-installed = Sin instalar: sus acciones en esta máquina esperan a INSTALAR.
settings-user-plugins-settings-form = AJUSTES DE ESTE PLUGIN
settings-user-plugins-save = GUARDAR
plugin-page-just-now = ahora mismo
plugin-page-minutes-ago = hace { $n } min
plugin-page-hours-ago = hace { $n } h
plugin-page-not-read-yet = Aún sin leer: pulsa ACTUALIZAR.
plugin-page-reading = Leyendo…
plugin-page-exit = código de salida { $code }
plugin-page-unreadable = No se puede leer la salida: { $why }
plugin-page-no-rows = Nada que mostrar.
plugin-page-runs = EJECUCIONES EN ESTE SERVIDOR
plugin-page-no-runs = Aún no hay ejecuciones.
plugin-page-running = { $label }…
plugin-page-truncated = Salida cortada en 1 MB.
plugin-page-from = { $name } { $version } · un plugin
plugin-page-refresh = ACTUALIZAR
plugin-page-not-accepted = Este plugin no está aceptado, o sus archivos cambiaron desde entonces. Revísalo en Ajustes → Plugins.
plugin-page-dry-run-chip = ENSAYO
plugin-page-stopped-chip = DETENIDO
plugin-page-running-now = En marcha…
plugin-page-stopping = DETENIENDO…
plugin-page-stop = DETENER
plugin-page-danger-title = { $label }: puede cortar el acceso o perder datos
plugin-page-change-title = { $label }: esto cambia el servidor
plugin-page-dry-run = ENSAYO · { $label }
plugin-page-dry-run-needed = Hazlo primero; EJECUTAR usa los mismos valores.
plugin-page-type-to-confirm = Escribe { $keyword } para confirmar
plugin-page-run = EJECUTAR · { $label }
plugin-page-cancel = CANCELAR
plugin-page-needs-install = Este plugin ejecuta cosas en esta máquina y necesita su instalación primero.
plugin-fleet-where = DÓNDE
plugin-fleet-count = { $count ->
    [one] 1 servidor
   *[other] { $count } servidores
}
plugin-local-lab-container = { $name } es un contenedor de laboratorio: las acciones en esta máquina llegan a los servidores por SSH, y no tiene.
plugin-run-login-before = Crow no puede iniciar sesión en este servidor antes del cambio, así que no se ejecutó nada: { $why }
plugin-run-login-after = El cambio se ejecutó, pero Crow no puede iniciar sesión después: { $why }. Vuelve a entrar con la consola de tu proveedor o restaura una instantánea.

## settings-providers
settings-providers-providers-where-your-servers-live = PROVEEDORES · DÓNDE VIVEN TUS SERVIDORES
settings-providers-set-up = { $configured }/{ $count } CONFIGURADOS
settings-providers-import-instances = IMPORTAR INSTANCIAS
settings-providers-tokens-cant-be-saved-right = Ahora mismo no se pueden guardar tokens: { $why }.
settings-providers-no-cloud-provider-plugin-is = No hay ningún plugin de proveedor de nube activado. Activa Linode o UpCloud en Ajustes → Plugins para conectar una cuenta.
settings-providers-connect-the-cloud-accounts-your = Conecta las cuentas de nube donde corren tus servidores. Crow lee tus instancias desde ellas y, si el proveedor lo permite, puede encender y apagar servidores y hacer instantáneas sin pasar por SSH. Los tokens se guardan cifrados en la bóveda de Crow: su clave la guarda el llavero del sistema (Keychain, Secret Service) o, con contraseña de bóveda, la bloquea esa contraseña. Llegan al proveedor por la entrada estándar de curl, nunca por la línea de comandos.
settings-providers-vault-security = Bóveda y seguridad →
settings-providers-not-set-up = sin configurar
settings-providers-testing = probando…
settings-providers-connected = conectado · { $msg }
settings-providers-saved-not-tested-yet = guardado · aún sin probar
settings-providers-checked = comprobado { $short_time }
settings-providers-edit = EDITAR
settings-providers-set-up-2 = CONFIGURAR
settings-providers-test-connection = PROBAR CONEXIÓN
settings-providers-click-again-to-remove = PULSA OTRA VEZ PARA QUITAR
settings-providers-remove = QUITAR
settings-providers-crow-is-open-anyone-at = Crow está ABIERTO: cualquiera en esta sesión puede usar este token para apagar servidores o hacerles instantáneas. Mira la insignia de postura.
settings-providers-not-saved = No guardado: { $e }
settings-providers-secret-vault = secreto · bóveda
settings-providers-clear = BORRAR
settings-providers-save-test = GUARDAR Y PROBAR
settings-providers-cancel = CANCELAR

## settings-vault-manage
settings-vault-manage-password = CONTRASEÑA
settings-vault-manage-change-password = CAMBIAR CONTRASEÑA…
settings-vault-manage-go-back-to-open = VOLVER A ABIERTO…
settings-vault-manage-going-back-to-open-means = Volver a ABIERTO significa:
settings-vault-manage-i-understand-without-the-password = Lo entiendo: sin la contraseña, cualquier cosa que se ejecute como yo puede leer los secretos de Crow.
settings-vault-manage-change-password-2 = CAMBIAR CONTRASEÑA
settings-vault-manage-go-open = PASAR A ABIERTO
settings-vault-manage-cancel = CANCELAR

## terminal
terminal-ctrl-shift-t = Ctrl+Mayús+T
terminal-ctrl-shift-d = Ctrl+Mayús+D
terminal-ctrl-shift-s = Ctrl+Mayús+S
terminal-ctrl-shift-w = Ctrl+Mayús+W
terminal-enter-renames-esc-cancels-empty = Intro renombra · Esc cancela · vacío sigue a la shell
terminal-press-enter-to-start-a = { $why } · pulsa Intro para abrir otra
terminal-starting = iniciando…
terminal-lines-back-type-to-return = ↑ { $scrolled } líneas atrás · escribe para volver
terminal-no-terminals-open-opens-one = No hay terminales abiertas. { $shortcut } abre una.

## topology
topology-crow = CROW
topology-server = SERVIDOR
topology-multipass-vm = VM DE MULTIPASS
topology-lab-container = CONTENEDOR DE LABORATORIO
topology-container = CONTENEDOR
topology-ssh-key = CLAVE SSH
topology-checking-posture = comprobando la postura…
topology-posture-not-checked-yet = postura aún sin comprobar
topology-just-now = ahora mismo
topology-min = { $mins } min
topology-all = TODO
topology-at-risk-only-crit-warn = SOLO EN RIESGO · { $crit } críticos · { $warn } avisos
topology-check-posture = COMPROBAR POSTURA
topology-reset-view = RESTABLECER VISTA
topology-no-servers-yet-enroll-one = Todavía no hay servidores: añade uno o lanza una VM de Multipass desde LABORATORIO LOCAL Y VMS.
topology-ssh = SSH
topology-via-jump-host = mediante host de salto
topology-key-unlocks = la clave abre
topology-critical-risk = riesgo crítico
topology-text = { $count } { $count ->
        [one] RIESGO
       *[other] RIESGOS
    }
topology-no-risks-found = NO SE ENCONTRARON RIESGOS
topology-risks = RIESGOS
topology-env-group = entorno · grupo
topology-host-key = clave de host
topology-not-pinned = sin fijar
topology-cpu-mem-disk = cpu { $cpu_pct }% · mem { $mem_pct }% · disco { $disk_pct }%
topology-posture = POSTURA
topology-not-checked-yet-check-posture = Aún sin comprobar (COMPROBAR POSTURA la lee en los servidores accesibles).
topology-couldnt-read-needs-root = no se pudo leer (necesita root)
topology-root-login = acceso de root
topology-open-to-all = abierto a todos
topology-key = CLAVE
topology-private-key = clave privada
topology-not-on-this-machine = no está en este equipo
topology-connected-to = CONECTADO A
topology-via-jump = mediante salto
topology-open-server = ABRIR SERVIDOR
topology-terminal = TERMINAL
topology-posture-checked-ago-servers = postura comprobada hace { $mins } · { $count } servidores

## users-host-data
users-host-data-cant-read-etc-passwd = no se puede leer /etc/passwd: { $e }
users-host-data-unnamed-key = clave sin nombre
users-host-data-is-not-a-valid-name = { $name } no es un nombre de { $kind } válido
users-host-data-the-password-is-empty = la contraseña está vacía
users-host-data-the-password-cant-contain-line = la contraseña no puede contener saltos de línea
users-host-data-the-password-is-longer-than = la contraseña tiene más de 256 caracteres
users-host-data-is-not-an-absolute-shell = { $shell } no es una ruta absoluta de shell
users-host-data-the-full-name-cant-contain = el nombre completo no puede contener ':' ni saltos de línea
users-host-data-the-root-account-cant-be = la cuenta root no se puede eliminar
users-host-data-not-a-valid-openssh-public = no es una clave pública OpenSSH válida

## users-models
users-models-active = ACTIVA
users-models-locked = BLOQUEADA
users-models-system = SISTEMA
users-models-all = TODAS
users-models-login = CON ACCESO
users-models-sudo = SUDO

## users
users-users = USUARIOS
users-new-user = + USUARIO NUEVO
users-user = USUARIO
users-uid = UID
users-groups = GRUPOS
users-shell = SHELL
users-keys = CLAVES
users-status = ESTADO
users-couldnt-read-accounts = No se pudieron leer las cuentas: { $e }
users-reading-accounts = Leyendo las cuentas…
users-no-accounts-match-try-all = Ninguna cuenta coincide — prueba con TODAS o borra la búsqueda
users-select-an-account-to-see = Selecciona una cuenta para ver sus detalles.
users-running = Ejecutando: { $p } …
users-sudo = SUDO
users-root = ROOT
users-account = CUENTA
users-uid-gid = uid / gid
users-can-log-in = puede entrar
users-no-password-locked = no — contraseña bloqueada

users-no-no-login-shell = no — sin shell de acceso
users-primary = { $g } (principal)
users-remove-from-no-sudo = QUITAR DE { $group } (SIN SUDO)
users-make-admin-add-to-sudo = HACER ADMINISTRADOR: AÑADIR A { $group } (SUDO)
users-authorized-ssh-keys = CLAVES SSH AUTORIZADAS
users-no-keys-in-ssh-authorized = No hay claves en ~/.ssh/authorized_keys.
users-add-key-from-vault = + AÑADIR CLAVE DE LA BÓVEDA
users-login-shell = SHELL DE ACCESO
users-password = CONTRASEÑA
users-locked-no-password-login-setting = bloqueada — sin acceso con contraseña (poner una la desbloquea)
users-set-can-log-in-with = definida (puede entrar con contraseña)
users-set-with-chpasswd-over-stdin = Se define con chpasswd por la entrada estándar; Crow no la guarda.
users-set-password = PONER CONTRASEÑA
users-cancel = CANCELAR
users-set-password-2 = PONER CONTRASEÑA…
users-actions = ACCIONES
users-unlock-password = DESBLOQUEAR CONTRASEÑA
users-lock-password = BLOQUEAR CONTRASEÑA
users-view-passwd-entry = VER ENTRADA DE PASSWD
users-click-again-to-delete = PULSA OTRA VEZ PARA ELIMINAR A { $username }
users-delete-user = ELIMINAR USUARIO

## users-new-user-modal
users-new-user-modal-provision-new-system-user = CREAR UN USUARIO DEL SISTEMA NUEVO
users-new-user-modal-login-username-posix = NOMBRE DE USUARIO (POSIX)
users-new-user-modal-full-name-gecos-metadata = NOMBRE COMPLETO (METADATOS GECOS)
users-new-user-modal-password-optional = CONTRASEÑA (OPCIONAL)
users-new-user-modal-the-passwords-dont-match = Las contraseñas no coinciden.
users-new-user-modal-leave-empty-for-key-only = Déjala vacía para acceder solo con clave (añade una clave desde el inspector después).
users-new-user-modal-set-with-chpasswd-over-stdin = Se define con chpasswd por la entrada estándar; Crow no la guarda.
users-new-user-modal-login-shell = SHELL DE ACCESO
users-new-user-modal-initial-privileges = PRIVILEGIOS INICIALES
users-new-user-modal-admin-group = ✓ ADMINISTRADOR (grupo { $g })
users-new-user-modal-standard-user-click-to-add = USUARIO ESTÁNDAR · pulsa para añadirlo a { $g }
users-new-user-modal-supplementary-groups-usermod-ag = GRUPOS ADICIONALES (usermod -aG)
users-new-user-modal-create-user-home-directory-home = Crear la carpeta personal (/home/<usuario>) con los archivos de configuración base
users-new-user-modal-enabled = ACTIVADO
users-new-user-modal-disabled = DESACTIVADO
users-new-user-modal-provision-account = CREAR CUENTA

## users-passwd-inspector
users-passwd-inspector-return-to-users = ← VOLVER A USUARIOS
users-passwd-inspector-account = CUENTA: { $uname }
users-passwd-inspector-open-in-config-editor = ABRIR EN EL EDITOR DE CONFIGURACIÓN
users-passwd-inspector-raw-7-field-posix-passwd = REGISTRO PASSWD POSIX DE 7 CAMPOS EN BRUTO
users-passwd-inspector-delimiter-colon = DELIMITADOR: ':' (DOS PUNTOS)
users-passwd-inspector-exploded-field-anatomy-system-semantics = ANATOMÍA DE LOS CAMPOS Y SU SIGNIFICADO
users-passwd-inspector-switch-passwd-record-to-inspect = CAMBIAR EL REGISTRO PASSWD A INSPECCIONAR
users-passwd-inspector-login-username = NOMBRE DE USUARIO
users-passwd-inspector-password-placeholder = MARCADOR DE CONTRASEÑA
users-passwd-inspector-indicates-account-password-is-locked = '!' indica que la contraseña de la cuenta está bloqueada. Se deniega el acceso por PAM/contraseña.
users-passwd-inspector-x-indicates-hashed-passwords-and = 'x' indica que los hashes y la sal de las contraseñas están en /etc/shadow, accesible solo para root (0640).
users-passwd-inspector-user-identifier-uid = IDENTIFICADOR DE USUARIO (UID)
users-passwd-inspector-uid-0-superuser-root-with = UID 0: superusuario (root) con privilegios sin restricciones en el kernel de Linux.
users-passwd-inspector-system-service-uid-reserved-for = UID de servicio del sistema: reservado para servicios en segundo plano aislados y servicios del sistema.
users-passwd-inspector-interactive-login-uid-standard-regular = UID de acceso interactivo: las cuentas normales de personas empiezan en el UID 1000+.
users-passwd-inspector-primary-group-gid = GRUPO PRINCIPAL (GID)
users-passwd-inspector-primary-numeric-group-id-applied = ID numérico del grupo principal que se aplica a los archivos y procesos nuevos de este usuario. Se define en /etc/group.
users-passwd-inspector-gecos-metadata = METADATOS GECOS
users-passwd-inspector-empty = (vacío)
users-passwd-inspector-general-electric-comprehensive-operating-supervisor = Campo heredado de General Electric Comprehensive Operating Supervisor. Guarda el nombre completo, la oficina y los teléfonos.
users-passwd-inspector-home-directory = CARPETA PERSONAL
users-passwd-inspector-initial-working-directory-set-upon = Carpeta de trabajo inicial al empezar una sesión. Contiene ~/.ssh, ~/.config y los archivos de configuración.
users-passwd-inspector-login-shell = SHELL DE ACCESO
users-passwd-inspector-prohibits-interactive-terminal-access-daemons = Impide el acceso interactivo a una terminal. Los servicios y cuentas de servicio usan nologin como defensa en profundidad.
users-passwd-inspector-command-interpreter-spawned-upon-login = Intérprete de comandos que se abre al iniciar sesión (p. ej. /bin/bash, /bin/zsh, /usr/bin/fish).
users-passwd-inspector-portion = PARTE N.º { $num }

## users-ssh-attach-modal
users-ssh-attach-modal-ssh-keychain-attachment = VINCULAR CLAVES SSH · { $uname }
users-ssh-attach-modal-attached = { $count } VINCULADAS
users-ssh-attach-modal-target-file = ARCHIVO DE DESTINO: { $target_auth_path }
users-ssh-attach-modal-permissions-0600-secure = PERMISOS: 0600 (SEGURO)
users-ssh-attach-modal-currently-authorized-keys-for-user = CLAVES AUTORIZADAS AHORA PARA EL USUARIO
users-ssh-attach-modal-no-ssh-keys-currently-installed = Este usuario no tiene ninguna clave SSH instalada. Vincula una abajo.
users-ssh-attach-modal-revoke = REVOCAR
users-ssh-attach-modal-select-from-crow-ssh-keychain = ELEGIR DEL LLAVERO SSH DE CROW
users-ssh-attach-modal-enrolled-vault-keys = { $count } CLAVES AÑADIDAS A LA BÓVEDA
users-ssh-attach-modal-no-enrolled-keys-found-in = No hay claves añadidas en la bóveda de Crow. Ve a Ajustes > Claves SSH para generar o analizar claves.
users-ssh-attach-modal-attached-2 = ✓ VINCULADA
users-ssh-attach-modal-attach-key = + VINCULAR CLAVE

## host-diagnose
host-diagnose-asks-to-approve-each-use = { $agent } (pide aprobar cada uso de una clave)
host-diagnose-is-the-ssh-agent-here = { $agent } es aquí el agente SSH y pide aprobar cada conexión nueva. Crow se conecta en segundo plano, así que una aprobación que nadie da parece un tiempo de espera agotado o un rechazo. Apruébala cuando { $agent } la pida, o configura { $agent } para que recuerde las aprobaciones de Crow.
host-diagnose-for-no-prompts-at-all = Para no tener ninguna pregunta, cambia este servidor a la propia clave de Crow (Claves → cambiar el servidor a una clave): Crow deja entonces fuera a { $agent }.
host-diagnose-ssh-said = ssh dijo
host-diagnose-jump-host = host de salto { $j }
host-diagnose-missing-on-this-machine = { $path } (no está en este equipo)
host-diagnose-your-ssh-agent-ssh-config = tu agente SSH / ~/.ssh/config
host-diagnose-connected = CONECTADO
host-diagnose-crow-reaches = Crow llega a { $addr }.
host-diagnose-key-tried = clave probada
host-diagnose-server-accepts = el servidor acepta
host-diagnose-the-key-file-isnt-on = El archivo de clave { $path } ya no está en este equipo: restáuralo o cambia este servidor a otra clave.
host-diagnose-the-server-doesnt-accept-that = El servidor no acepta esa clave para { $login_user }: no está en ~{ $login_user }/.ssh/authorized_keys allí, o sshd ignora el archivo (permisos, AuthorizedKeysFile).

host-diagnose-the-server-doesnt-offer-key = El servidor no ofrece acceso con clave (PubkeyAuthentication no): Crow solo entra con claves.
host-diagnose-it-accepts-a-password-log = Acepta contraseña: entra una vez con ella y Crow instala su clave, sin guardar la contraseña.
host-diagnose-login-refused = ACCESO RECHAZADO
host-diagnose-answered-but-refused-crows-login = { $addr } respondió, pero rechazó el acceso de Crow.
host-diagnose-presents-now = presenta ahora
host-diagnose-cant-fetch-through-a-jump = no se puede obtener a través de un host de salto
host-diagnose-host-key-changed = CLAVE DE HOST CAMBIADA
host-diagnose-presents-a-different-host-key = { $addr } presenta una clave de host distinta de la que Crow tenía como de confianza. O el servidor se reinstaló o se regeneraron sus claves, o algo entre tú y él se está haciendo pasar por él.
host-diagnose-dont-trust-the-new-key = No confíes en la clave nueva hasta saber por qué cambió.
host-diagnose-check-it-on-the-server = Compruébala en el propio servidor (consola, shell web del proveedor): ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub, y compárala con «presenta ahora».
host-diagnose-host-key-not-trusted = CLAVE DE HOST SIN CONFIANZA
host-diagnose-ssh-known-hosts-has-no = ~/.ssh/known_hosts no tiene ninguna clave para { $addr }, así que ssh se niega a conectar (comprobación estricta).
host-diagnose-compare-the-fingerprint-with-the = Compara la huella con la de la consola del servidor antes de confiar en ella.
host-diagnose-this-server-is-behind-a = Este servidor está detrás de un host de salto, así que Crow no puede obtener su clave directamente: verifícala en el servidor y añádela tú a known_hosts.
host-diagnose-once-it-checks-out-trust = Cuando cuadre: confía en la clave nueva (escribes el nombre del servidor para confirmar).
host-diagnose-jump-host-failed = FALLÓ EL HOST DE SALTO
host-diagnose-the-connection-through-failed-before = La conexión a través de { $j } falló antes de llegar a { $addr }.
host-diagnose-open-in-crow-if-its = Abre { $j } en Crow: si también es inaccesible, arregla eso primero.
host-diagnose-check-the-jump-host-can = Comprueba que el host de salto llega a la dirección y el puerto de este servidor.
host-diagnose-name-not-found = NOMBRE NO ENCONTRADO
host-diagnose-doesnt-resolve-to-an-address = { $host } no se resuelve a ninguna dirección desde este equipo.
host-diagnose-check-the-spelling-or-use = Revisa cómo está escrito, o usa la dirección IP.
host-diagnose-a-vpn-or-private-dns = Puede hacer falta una VPN o un DNS privado para nombres internos.
host-diagnose-connection-refused = CONEXIÓN RECHAZADA
host-diagnose-answered-but-nothing-accepts-connections = { $host } respondió, pero nada acepta conexiones en el puerto { $server }.
host-diagnose-a-firewall-rejecting-the-port = Un cortafuegos que rechaza el puerto se ve igual.
host-diagnose-timed-out = TIEMPO AGOTADO
host-diagnose-didnt-answer-at-all = { $host } no respondió en absoluto.
host-diagnose-the-server-may-be-off = El servidor puede estar apagado o reiniciándose; compruébalo en su proveedor.
host-diagnose-a-firewall-dropping-the-port = Un cortafuegos que descarta el puerto, o una dirección equivocada, se ve igual.
host-diagnose-no-route = SIN RUTA
host-diagnose-this-machine-has-no-route = Este equipo no tiene ruta hasta { $host }.
host-diagnose-check-your-network-vpn-or = Revisa tu red, la VPN o la dirección.
host-diagnose-unreachable = INACCESIBLE
host-diagnose-crow-couldnt-open-a-connection = Crow no pudo abrir una conexión con { $addr }.
host-diagnose-try-again-if-it-persists = Inténtalo de nuevo; si sigue igual, conéctate por ssh desde una terminal para ver el error completo.
host-diagnose-direct-probe = sondeo directo
host-diagnose-no-ssh-banner = sin banner ssh
host-diagnose-no-answer = sin respuesta
host-diagnose-port-open-ms = puerto abierto ({ $p } ms) · { $p2 }

## host-ssh
host-ssh-connected = CONECTADO
host-ssh-auth-failed = FALLÓ LA AUTENTICACIÓN
host-ssh-host-key = CLAVE DE HOST
host-ssh-unreachable = INACCESIBLE
host-ssh-1password = 1Password
host-ssh-secretive = Secretive
host-ssh-login-failed = falló el acceso
host-ssh-isnt-a-private-folder-of = { $dir } no es una carpeta privada de este usuario
host-ssh-the-bastions-loop-back-to = los bastiones vuelven en bucle a { $id }: corrige uno de ellos en sus ajustes
host-ssh-more-than-bastions-deep = más de { $max_hops } bastiones de profundidad
host-ssh-its-bastion-is-no-longer = su bastión ({ $id }) ya no está en la flota
host-ssh-bastion-has-an-invalid-address = el bastión { $name } tiene una dirección no válida
host-ssh-bastion-logs-in-with-a = el bastión { $name } entra con contraseña; Crow pasa por los bastiones con claves
host-ssh-bastion-s-key-has-no = la clave del bastión { $name } no tiene archivo de clave privada
host-ssh-no-bastion = sin bastión
host-ssh-holds-a-which-ssh-would = { $bad } contiene un '%', que ssh interpretaría como un comodín
host-ssh-the-server-directory-is-busy = el directorio de servidores está ocupado
host-ssh-the-bastion = el bastión
host-ssh-bastion-cant-reach = el bastión { $via } no llega a { $target }: { $why }
host-ssh-bastion-refused-crows-key = el bastión { $name } rechazó la clave de Crow: { $line }
host-ssh-bastion = bastión { $name }: { $line }
host-ssh-this-server-is-set-to = Este servidor está configurado para acceso con contraseña; Crow se conecta con claves, que instala una vez con la contraseña.
host-ssh-the-enrolled-key-for-this = la clave añadida para este servidor no tiene archivo de clave privada
host-ssh-invalid-host-address = dirección de host no válida { $host }
host-ssh-via = {" "}mediante { $name }
host-ssh-bastion-2 = bastión { $name }:{" "}
host-ssh-bastion-refused-crows-key-2 = el bastión { $name } rechazó la clave de Crow: { $d }
host-ssh-ssh-failed = falló ssh

## security-hardening
security-hardening-root-can-log-in-over = root puede entrar por SSH con contraseña (PermitRootLogin yes)
security-hardening-root-can-log-in-over-2 = root puede entrar por SSH con una clave (PermitRootLogin { $v })
security-hardening-no-firewall-filters-incoming-traffic = ningún cortafuegos filtra el tráfico entrante
security-hardening-no-firewall-and-open-to = sin cortafuegos, y { $count ->
        [one] el puerto
       *[other] los puertos
    } { $p } { $count ->
        [one] abierto
       *[other] abiertos
    } a todas las redes
security-hardening-permitrootlogin-prohibit-password-crow-logs = PermitRootLogin prohibit-password (Crow entra como root con una clave, así que no «no»)
security-hardening-permitrootlogin-prohibit-password = PermitRootLogin prohibit-password
security-hardening-permitrootlogin-no = PermitRootLogin no
security-hardening-passwordauthentication-no-once-a-key = PasswordAuthentication no (una vez comprobado el acceso con clave)
security-hardening-and-the-listening = {" "}y { $count ->
        [one] el puerto
       *[other] los puertos
    } en escucha { $p }
security-hardening-crow-logs-in-to-this = ✕ Crow entra en este servidor como root: PermitRootLogin no dejaría fuera a Crow. No se cambió nada.
security-hardening-a-fresh-key-login-works = ✓ Funciona un inicio de sesión nuevo con clave.
security-hardening-this-sshd-config-doesnt-include = ✕ Este sshd_config no incluye sshd_config.d, así que un archivo adicional no tendría efecto. No se cambió nada; define PermitRootLogin en la hoja de sshd_config.
security-hardening-and-reloaded-sshd = {" "}y recargado sshd
security-hardening-undo-didnt-finish-remove-by = ⚠ No se terminó de deshacer: quita { $root_dropin } a mano y recarga sshd.

security-hardening-wrote = ✓ Escrito { $root_dropin }.
security-hardening-sshd-rejected-the-change = ✕ sshd rechazó el cambio: { $e }
security-hardening-sshd-t-accepts-it = ✓ sshd -t lo acepta.
security-hardening-sshd-will-use-permitrootlogin = ✓ sshd usará PermitRootLogin { $value }.
security-hardening-something-earlier-in-the-config = ✕ Algo anterior en la configuración sigue definiendo PermitRootLogin ({ $other }).
security-hardening-couldnt-check-the-result = ✕ No se pudo comprobar el resultado: { $e }
security-hardening-couldnt-reload-sshd = ✕ No se pudo recargar sshd: { $e }
security-hardening-reloaded-sshd = ✓ sshd recargado.
security-hardening-a-fresh-key-login-failed = ✕ Un inicio de sesión nuevo con clave falló tras la recarga ({ $e }).
security-hardening-a-fresh-key-login-still = ✓ Un inicio de sesión nuevo con clave sigue funcionando. PermitRootLogin es { $value }.
security-hardening-a-fresh-key-login-doesnt-2 = ✕ No funciona un inicio de sesión nuevo con clave ({ $e }). No se cambió nada.
security-hardening-neither-ufw-nor-firewalld-is = ✕ No está instalado ni ufw ni firewalld. Instala uno (apt install ufw, dnf install firewalld) y vuelve a intentarlo. No se cambió nada.
security-hardening-undone = ↺ Deshecho: { $off }.
security-hardening-undo-didnt-finish-run-by = ⚠ No se terminó de deshacer: ejecuta { $off } a mano.
security-hardening-a-fresh-key-login-failed-2 = ✕ Un inicio de sesión nuevo con clave falló con el cortafuegos activado ({ $e }).
security-hardening-a-fresh-key-login-still-2 = ✓ Un inicio de sesión nuevo con clave sigue funcionando. { $tool } está activado y permite { $ports }.
security-hardening-turn-a-firewall-on-that = activar un cortafuegos que permita SSH{ $p }
security-hardening-undone-removed = ↺ Deshecho: eliminado { $root_dropin }{ $reload }.

## security-stance
security-stance-open = ABIERTO
security-stance-locked = BLOQUEADO
security-stance-secrets-provider-tokens-ai-keys = Los secretos (tokens de proveedores, claves de IA) se cifran en crow.db, así que una copia del archivo o una copia de seguridad no los revela.
security-stance-tokens-never-appear-on-a = Los tokens nunca aparecen en una línea de comandos ni en los registros.
security-stance-everything-open-protects = Todo lo que protege Abierto.
security-stance-nobody-can-open-crow-or = Nadie puede abrir Crow ni usar sus secretos sin tu contraseña y un código 2FA, ni siquiera los programas que se ejecutan como tú.
security-stance-the-key-is-wiped-from = La clave se borra de la memoria cuando Crow se bloquea.
security-stance-anyone-at-your-unlocked-computer = Cualquiera ante tu equipo desbloqueado puede abrir Crow y usar todo lo que contiene.
security-stance-any-program-running-as-you = Cualquier programa que se ejecute como tú puede pedir la clave de Crow al llavero del sistema mientras tienes la sesión iniciada (tu sesión desbloquea el llavero), y leer así todos los secretos.
security-stance-provider-tokens-can-power-servers = Los tokens de proveedores pueden apagar servidores y hacer instantáneas; quien tenga tu sesión tiene ese poder.
security-stance-crows-ssh-keys-in-ssh = Las claves SSH de Crow en ~/.ssh son archivos que Crow no cifra: salvo que tengan frase de paso, cualquier cosa que se ejecute como tú puede usarlas para llegar a tus servidores.
security-stance-while-crow-is-unlocked-anyone = Mientras Crow está desbloqueado, cualquiera ante tu equipo puede usarlo, hasta que se bloquee solo.
security-stance-crows-ssh-keys-in-ssh-2 = Las claves SSH de Crow en ~/.ssh son archivos que Crow no cifra: salvo que tengan frase de paso, cualquier cosa que se ejecute como tú puede usarlas, esté bloqueado o no.
security-stance-your-server-list-settings-and = Tu lista de servidores, tus ajustes y el historial de cambios no se cifran en ninguna de las dos posturas.
security-stance-nothing-to-type-nothing-to = Nada que escribir, nada que perder. Esa comodidad es la exposición de arriba.
security-stance-you-unlock-crow-with-your = Desbloqueas Crow con tu contraseña y un código 2FA cada vez que lo abres y tras el bloqueo automático.
security-stance-lose-the-password-or-the = Si pierdes la contraseña o el autenticador, los secretos guardados se pierden para siempre. No hay restablecimiento ni recuperación.
security-stance-ai-api-keys-are-still = Las claves de API de IA siguen guardadas en texto plano (de una versión antigua de Crow). Pasan a la bóveda en cuanto haya una clave de cifrado.
security-stance-no-os-keyring-is-available = No hay llavero del sistema disponible ({ $why }), así que Crow no puede guardar secretos. Cambia a Bloqueado para guardarlos.
security-stance-provider-can-power-servers-or = { $action_capable_accounts } { $action_capable_accounts ->
        [one] cuenta de proveedor puede
       *[other] cuentas de proveedor pueden
    } encender y apagar servidores o hacer instantáneas, al alcance de cualquiera en esta sesión.
security-stance-auto-lock-is-off-once = El bloqueo automático está desactivado: una vez desbloqueado, Crow sigue así hasta que lo bloquees o lo cierres.
security-stance-auto-lock-is-minutes-an = El bloqueo automático es de { $auto_lock_minutes } minutos: un Crow desbloqueado y desatendido se puede usar durante ese tiempo.

## security-sshd-passwords
security-sshd-passwords-a-fresh-key-login-works = ✓ Funciona un inicio de sesión nuevo con clave.
security-sshd-passwords-password-login-is-already-off = ✓ El acceso con contraseña ya está desactivado. No hay nada que hacer.
security-sshd-passwords-this-sshd-config-doesnt-include = ✕ Este sshd_config no incluye sshd_config.d, así que un archivo adicional no tendría efecto. No se cambió nada; pon PasswordAuthentication en no en la hoja de sshd_config.
security-sshd-passwords-and-reloaded-sshd = {" "}y recargado sshd
security-sshd-passwords-undo-didnt-finish-remove-by = ⚠ No se terminó de deshacer: quita { $dropin } a mano y recarga sshd.
security-sshd-passwords-wrote = ✓ Escrito { $dropin }.
security-sshd-passwords-sshd-rejected-the-change = ✕ sshd rechazó el cambio: { $e }
security-sshd-passwords-sshd-t-accepts-it = ✓ sshd -t lo acepta.
security-sshd-passwords-sshd-will-refuse-passwords = ✓ sshd rechazará las contraseñas.
security-sshd-passwords-set-in = {" "}(definido en: { $who })
security-sshd-passwords-couldnt-check-the-result = ✕ No se pudo comprobar el resultado: { $e }
security-sshd-passwords-couldnt-reload-sshd = ✕ No se pudo recargar sshd: { $e }
security-sshd-passwords-reloaded-sshd = ✓ sshd recargado.
security-sshd-passwords-a-fresh-key-login-failed = ✕ Un inicio de sesión nuevo con clave falló tras la recarga ({ $e }).
security-sshd-passwords-a-fresh-key-login-still = ✓ Un inicio de sesión nuevo con clave sigue funcionando. El acceso con contraseña está desactivado.
security-sshd-passwords-undone-removed = ↺ Deshecho: eliminado { $dropin }{ $reload }.
security-sshd-passwords-something-else-still-turns-password = ✕ Algo más sigue activando el acceso con contraseña{ $who }.

## security-host-keys
security-host-keys-this-server-uses-host-certificates = este servidor usa certificados de host; rótalos con la CA que los firma
security-host-keys-couldnt-read-sshds-settings = no se pudieron leer los ajustes de sshd: { $e }
security-host-keys-couldnt-generate-new-keys = no se pudieron generar claves nuevas: { $e }
security-host-keys-none-of-sshds-host-keys = ninguna de las claves de host de sshd existe en disco
security-host-keys-the-server-returned-a-public = el servidor devolvió una clave pública que Crow no pudo leer; no se cambió nada
security-host-keys-the-new-keys-werent-put = las claves nuevas no se colocaron: { $e }
security-host-keys-rolled-back-the-old-host = { $why }. Revertido: las claves de host antiguas vuelven a estar y known_hosts está como estaba.

security-host-keys-server = servidor: { $e }
security-host-keys-known-hosts-couldnt-be-updated = no se pudo actualizar known_hosts ({ $e })
security-host-keys-a-fresh-login-with-the = falló un inicio de sesión nuevo con la clave nueva ({ $e })
security-host-keys-rollback-didnt-finish-the-old = { $why }. No se terminó de revertir ({ $r }{ $l }); las claves antiguas se guardan en { $backup } en el servidor.

## security-osv
security-osv-osv-answered-with-something-unexpected = OSV respondió algo inesperado: { $e }
security-osv-an-unknown-os = un sistema desconocido
security-osv-cve-lookup-covers-debian-and = La búsqueda de CVE cubre Debian y Ubuntu; este servidor usa { $report }

## security-sessions
security-sessions-crow-couldnt-find-its-own = Crow no encontró su propia conexión entre las de sshd, así que no cerró nada
security-sessions-no-other-ssh-logins = no hay otras sesiones SSH
security-sessions-ended-other-ssh = { $count ->
        [one] cerrada { $count } sesión SSH más
       *[other] cerradas { $count } sesiones SSH más
    }: { $who }

## lab-multipass
lab-multipass-install-multipass-with-homebrew = Instalar Multipass con Homebrew
lab-multipass-install-multipass-from-canonical = Instalar Multipass desde Canonical
lab-multipass-multipass-ships-as-a-snap = Multipass se distribuye como snap; añade snapd a la imagen del sistema y reinicia
lab-multipass-multipass-ships-as-a-snap-2 = Multipass se distribuye como snap; instala snapd con tu gestor de paquetes
lab-multipass-start-snapd = Iniciar snapd
lab-multipass-install-multipass = Instalar Multipass
lab-multipass-let-multipasss-dns-server-find = Que el servidor DNS de Multipass encuentre el usuario nobody (en este sistema los usuarios del sistema están en /usr/lib)
lab-multipass-let-vms-get-an-address = Que las VMs obtengan dirección: firewalld bloquea DHCP y DNS en el puente privado de Multipass
lab-multipass-restart-the-multipass-daemon = Reiniciar el servicio de Multipass
lab-multipass-the-daemon-didnt-answer = el servicio no respondió
lab-multipass-a-vm-name-is-a = { $name }: el nombre de una VM es una letra seguida de letras, dígitos y guiones (sin acabar en guion)
lab-multipass-a-vm-needs-at-least = una VM necesita al menos 1 CPU, 1 GB de memoria y 5 GB de disco
lab-multipass-isnt-an-image-name-multipass = { $image } no es un nombre de imagen que Multipass conozca
lab-multipass-crows-public-key-isnt-a = la clave pública de Crow no es una clave OpenSSH válida
lab-multipass-couldnt-pass-the-cloud-init = no se pudo pasar el cloud-init: { $e }
lab-multipass-unexpected-multipass-list-output = salida inesperada de `multipass list`: { $e }
lab-multipass-unexpected-multipass-info-output = salida inesperada de `multipass info`: { $e }
lab-multipass-has-no-ssh-host-keys = { $name } todavía no tiene claves de host SSH; ¿sigue arrancando?
lab-multipass-not-a-valid-openssh-public = no es una clave pública OpenSSH válida
lab-multipass-start = Iniciar
lab-multipass-stop = Detener
lab-multipass-restart = Reiniciar
lab-multipass-suspend = Suspender
lab-multipass-delete-for-good = Eliminar definitivamente
lab-multipass-not-running-yet = aún no está en marcha
lab-multipass-no-address-yet = aún sin dirección
lab-multipass-state = estado { $state }
lab-multipass-didnt-come-up-in-s = { $name } no arrancó en { $as_secs } s ({ $last })
lab-multipass-no-host-keys-to-pin = no hay claves de host que fijar
lab-multipass-couldnt-write-ssh-known-hosts = no se pudo escribir ~/.ssh/known_hosts: { $e }
lab-multipass-waiting-for-an-address-and = Esperando una dirección y las claves de host
lab-multipass-pinning-host-keys-for = Fijando las claves de host de { $address }
lab-multipass-logging-in-as-ubuntu-with = Entrando como ubuntu con la clave de Crow
lab-multipass-reading-its-facts = Leyendo sus datos
lab-multipass-adding-crows-key-to = Añadiendo la clave de Crow a { $name }

## journal
journal-emerg = EMERG
journal-alert = ALERTA
journal-notice = AVISO
journal-debug = DEPURACIÓN
journal-live = EN DIRECTO
journal-all-time = TODO
journal-this-boot = ESTE ARRANQUE
journal-previous-boot = ARRANQUE ANTERIOR
journal-just-now = ahora mismo
journal-s-ago = hace { $diff } s
journal-m-ago = hace { $diff } min
journal-h-ago = hace { $diff } h
journal-d-ago = hace { $diff } d

## dr
dr-min = { $s } min
dr-days = { $s } días
dr-no-backup-found-at = no se encontró ninguna copia de seguridad en { $path }
dr-last-ran = , última ejecución { $last }
dr-not-found = no encontrado
dr-exists = { $p } existe
dr-isnt-there-the-plan-may = { $p } no está: puede que el plan esté desactualizado
dr-config-pinned-as-baselines-rebuildable = { $baselines } { $baselines ->
        [one] archivo de configuración fijado
       *[other] archivos de configuración fijados
    } como línea base (reconstruibles desde Crow)
dr-no-config-file-pinned-as = ningún archivo de configuración fijado como línea base: fija los que necesite una reconstrucción
dr-can-snapshot-it-danger-zone = { $p } puede hacerle instantáneas (Zona de peligro → INSTANTÁNEAS)
dr-the-plan-names-no-backups = el plan no nombra ninguna copia de seguridad
dr-is-active = { $t } está activo{ $last }
dr-isnt-running-backups-wont-be = { $t } no está en marcha ({ $state }): no se harán copias de seguridad

## patching
patching-security-updates = actualizaciones de seguridad
patching-all-updates = todas las actualizaciones
patching-s-package-index-was-never = el índice de paquetes de { $manager } nunca se descargó, así que no se conocen las actualizaciones pendientes: ejecuta allí `{ $manager } update`
patching-nothing-to-upgrade = nada que actualizar
patching-isnt-a-package-name = { $bad } no es un nombre de paquete
patching-no-package-manager-crow-knows = ningún gestor de paquetes que Crow conozca (apt, dnf, apk)
patching-isnt-a-package-manager-crow = { $other } no es un gestor de paquetes con el que Crow pueda parchear
patching-upgraded = { $count } actualizados
patching-still-pending = , { $count } aún pendientes ({ $still_pending })
patching-reboot-required = {" "}· hace falta reiniciar
patching-failed-exit = { $manager } falló (salida { $status }): { $last_lines }
patching-a-window-looks-like-sun = una ventana tiene la forma «Sun 02-05» o «Mon-Fri 22-06»
patching-isnt-a-day = { $a } no es un día
patching-isnt-a-day-2 = { $b } no es un día
patching-isnt-a-day-3 = { $part } no es un día
patching-no-days-in-the-window = no hay días en la ventana
patching-hours-look-like-02-05 = las horas tienen la forma 02-05
patching-hours-are-0-to-24 = las horas van de 0 a 24
patching-the-windows-start-and-end = el inicio y el fin de la ventana son la misma hora
patching-outside-s-maintenance-window = fuera de la ventana de mantenimiento de { $group } ({ $describe })
patching-no-systemd-to-wait-for = no hay systemd al que esperar
patching-services-up = servicios en marcha ({ $state })
patching-came-back-but-systemd-was = volvió, pero systemd seguía arrancando pasados { $as_secs } s
patching-came-back-but-failed = volvió, pero { $count ->
        [one] una unidad falló
       *[other] fallaron unidades
    }: { $units }

## certs
certs-certbots-timer-isnt-running-certificates = el temporizador de certbot no está en marcha ({ $t }): los certificados no se renovarán solos
certs-certbots-last-renewal-failed = la última renovación de certbot falló: { $last }
certs-exit = salida { $status }: { $stderr }
certs-isnt-a-certificate-name = ✕ { $name } no es un nombre de certificado.
certs-dry-run-passed = ✓ La prueba en seco pasó: { $tail }
certs-the-dry-run-failed-so = ✕ La prueba en seco falló, así que no se renovó nada: { $e }
certs-not-due-yet-certbot-renews = = Aún no toca: certbot renueva a menos de 30 días de la caducidad, y Let's Encrypt limita cuántas veces se puede emitir un certificado.

certs-renewed = ✓ Renovado: { $tail }
certs-renewal-failed = ✕ La renovación falló: { $e }
certs-isnt-a-domain = ✕ { $domain } no es un dominio.
certs-acme-sh-has-no-dry = · acme.sh no tiene prueba en seco: se renueva directamente.
certs-not-due-yet-acme-sh = = Aún no toca (acme.sh renueva 60 días después de emitirlo).
certs-nginx-config-checked-and-reloaded = ✓ Configuración de nginx comprobada y recargada.
certs-nginx-wasnt-reloaded-it-still = ✕ nginx no se recargó ({ $e }): sigue sirviendo el certificado antiguo hasta que se recargue.
certs-apache-config-checked-and-reloaded = ✓ Configuración de Apache comprobada y recargada.
certs-apache-wasnt-reloaded-it-still = ✕ Apache no se recargó ({ $e }): sigue sirviendo el certificado antiguo hasta que se recargue.
certs-the-certificate-now-expires = ✓ El certificado caduca ahora el { $expiry }.
certs-couldnt-re-read-the-certificate = · No se pudo volver a leer el certificado para confirmar su nueva caducidad.

## keys
keys-ed25519-recommended = Ed25519 (recomendado)
keys-rsa-4096-bit = RSA de 4096 bits
keys-failed-to-create-destination-directory = No se pudo crear la carpeta de destino «{ $target_dir }»: { $e }
keys-failed-to-generate-ed25519-key = No se pudo generar la clave Ed25519: { $e }
keys-failed-to-generate-rsa-4096 = No se pudo generar la clave RSA de 4096 bits: { $e }
keys-failed-to-format-public-key = No se pudo dar formato a la clave pública: { $e }
keys-failed-to-format-private-key = No se pudo dar formato a la clave privada: { $e }
keys-the-keys-file-name-cant = El nombre de archivo de la clave no puede contener / ni ser . o ..
keys-public-key-file-already-exists = Ya existe un archivo de clave pública en «{ $pub_path }»
keys-private-key-file-already-exists = Ya existe un archivo de clave privada en «{ $priv_path }»
keys-failed-to-write-private-key = No se pudo escribir la clave privada en «{ $priv_path }»: { $e }
keys-failed-to-write-public-key = No se pudo escribir la clave pública en «{ $pub_path }»: { $e }

## keys-deploy
keys-deploy-the-new-key-isnt-an = la clave nueva no es una clave pública OpenSSH
keys-deploy-has-no-home-directory-on = { $user } no tiene carpeta personal en el servidor
keys-deploy-the-new-key-didnt-log = la clave nueva no pudo entrar ({ $e }); se quitó de nuevo y no se cambió nada

## host-bootstrap
host-bootstrap-is-not-a-valid-host = { $host } no es un host válido
host-bootstrap-cant-locate-the-crow-binary = no se encuentra el binario de Crow para askpass: { $e }
host-bootstrap-couldnt-run-ssh = no se pudo ejecutar ssh: { $version }
host-bootstrap-askpass-socket = socket de askpass: { $e }
host-bootstrap-the-key-install-script-didnt = el script de instalación de la clave no terminó: { $stderr }
host-bootstrap-installing-the-key-failed = falló la instalación de la clave: { $stderr }
host-bootstrap-timed-out-logging-in-with = se agotó el tiempo al entrar con la contraseña

## topology
topology-its-health-check-fails = su comprobación de salud falla
topology-stopped-with-an-error = detenido con un error: { $status }
topology-restarted-times = reiniciado { $n } veces
topology-port-published-to-every-network = puerto { $host_port } (→ { $container }) publicado a todas las redes
topology-dsa-key-broken-and-refused = clave DSA: rota, y OpenSSH actual la rechaza
topology-rsa-bit-under-the-3072 = RSA de { $bits } bits: por debajo de los 3072 bits que piden las recomendaciones actuales
topology-crow-2 = Crow
topology-this-machine = este equipo
topology-host-key-not-pinned-crow = clave de host sin fijar: Crow nunca registró qué clave debería tener este servidor
topology-crow-logs-in-with-a = Crow entra con contraseña, no con clave
topology-jump-host-for-servers-if = host de salto de { $n } servidores: si cae, Crow no puede llegar a ellos
topology-servers = SERVIDORES
topology-days-old-past-the-day = { $days } días: supera la recomendación de rotar cada { $key_max_age_days } días
topology-no-private-key-file-recorded = no hay archivo de clave privada registrado en este equipo
topology-keys = CLAVES

## team
team-restart-services = reiniciar servicios
team-edit-configs = editar configuraciones
team-fleet-runs = ejecuciones de flota
team-users-firewall = usuarios y cortafuegos
team-ssh-keys = claves SSH
team-vault-settings = bóveda y ajustes
team-viewer = Observador
team-operator = Operador
team-admin = Administrador
team-the-whole-fleet = toda la flota
team-text = { $count ->
        [one] grupo
       *[other] grupos
    } { $g }
team-sign-in-first-this-crow = inicia sesión primero: este Crow tiene un equipo
team-is-a-needs-a-role = { $name } es { $name2 }: { $label } necesita un rol que permita { $label }
team-isnt-in-s-scope = { $name } no está en el ámbito de { $name2 } ({ $describe })
team-a-passphrase-needs-at-least = una frase de paso necesita al menos 8 caracteres

## update
update-this-crow-runs-from-rather = este Crow se ejecuta desde { $exe } y no desde un Crow.app instalado, así que no se sustituye
update-no-home-directory = no hay carpeta personal
update-this-crow-runs-from-rather-2 = este Crow se ejecuta desde { $exe } y no desde el instalado { $installed }, así que no se sustituye (actualiza tu compilación desde el código)
update-theres-no-release-build-for = no hay versión publicada para esta plataforma
update-has-no = { $tag } no tiene { $name }
update-has-no-provenance-intoto-jsonl = { $tag } no tiene procedencia (.intoto.jsonl), así que no se puede verificar
update-installing-updates-needs-slsa-verifier = instalar actualizaciones requiere slsa-verifier (github.com/slsa-framework/slsa-verifier) para comprobar la procedencia firmada de la versión; no está en el PATH. No se instaló nada
update-verification-failed = la verificación falló
update-crow-is-installed-verified-restart = Crow { $version } está instalado (verificado). Reinicia Crow para usarlo.
update-couldnt-move-the-old-crow = no se pudo apartar el Crow.app antiguo: { $e }
update-the-release-didnt-verify-so = la versión no pasó la verificación, así que no se instaló nada: { $from_utf8_lossy }

## providers
providers-isnt-compiled-into-this-build = { $plugin } no está incluido en esta compilación
providers-required = { $count ->
        [one] Falta
       *[other] Faltan
    } { $names }
providers-provider-secrets-are-only-stored = los secretos de proveedores solo se guardan cifrados, y no hay clave de cifrado disponible
providers-the-stored-isnt-text = el { $field } guardado no es texto
providers-cant-power-instances = { $name } no puede encender ni apagar instancias
providers-accepted-boot = { $name } aceptó: arrancar { $id }
providers-accepted-reboot = { $name } aceptó: reiniciar { $id }
providers-accepted-shut-down = { $name } aceptó: apagar { $id }
providers-cant-take-snapshots = { $name } no puede hacer instantáneas
providers-snapshot-started-for = instantánea { $id } de { $name } ({ $status }) iniciada para { $id2 }

## incident
incident-alert = ALERTA
incident-resolved = RESUELTA
incident-change = CAMBIO
incident-check = COMPROBACIÓN
incident-log = REGISTRO

## config-push
config-push-removed-it-didnt-exist-before = eliminado (no existía antes)
config-push-removed-with-none-of-it = eliminado, con { $created_dirs } (nada de ello existía antes)
config-push-couldnt-read = no se pudo leer { $path }: { $e }
config-push-already-matches-the-baseline = ya coincide con la línea base
config-push-not-written = no escrito: { $e }
config-push-not-written-the-hosts-validator = no escrito: el validador del host rechazó la línea base: { $e }

config-push-the-vault-is-busy = la bóveda está ocupada
config-push-written = escrito ({ $differences } { $differences ->
        [one] diferencia
       *[other] diferencias
    })
config-push-history-not-recorded = { $what }; historial no registrado: { $e }

## config-drift
config-drift-missing = falta: { $r }
config-drift-extra = sobra: { $r }
config-drift-same-lines-in-a-different = mismas líneas en otro orden (en este archivo el orden importa)
config-drift-differs-from-the-baseline-only = difiere de la línea base (solo se guardaron hashes, así que Crow no puede decir en qué)
config-drift-group = grupo { $scope }
config-drift-more = {" "}(+{ $d } más)
config-drift-drifted-from-the-baseline = { $path } se desvió de la línea base de { $scope }: { $first }{ $more }

## metrics-certs
metrics-certs-certificate-for = certificado de { $name }
metrics-certs-expired-day-ago = caducó hace { $d } { $d ->
        [one] día
       *[other] días
    }
metrics-certs-expires-today = caduca hoy
metrics-certs-expires-tomorrow = caduca mañana
metrics-certs-expires-in-days = caduca en { $d } días

## host-reboot
host-reboot-not-rebooted-couldnt-read-the = no reiniciado: no se pudo leer antes la marca de arranque: { $e }
host-reboot-back-after-s = de vuelta tras { $as_secs } s
host-reboot-the-reboot-request-failed-and = la petición de reinicio falló ({ $e }) y el servidor no se reinició
host-reboot-not-back-after-s = sin volver tras { $as_secs } s

## rollout
rollout-couldnt-run-it-exit = no se pudo ejecutar (salida { $status }): { $last_lines }
rollout-done-no-output = hecho (sin salida)
rollout-ended-without-an-exit-code = terminó sin código de salida: { $tail }
rollout-exit = salida { $n }: { $tail }

## people
people-no-comment = sin comentario
people-crow-logs-in-to-as = Crow entra en { $server } como { $user }: darlo de baja dejaría fuera a Crow
people-root-isnt-offboarded-turn-off = root no se da de baja: desactiva el acceso SSH de root en ENDURECIMIENTO
people-no-account-on = no hay cuenta { $user } en { $server }
people-its-the-key-crow-logs = es la clave con la que Crow entra en { $server }: cambia antes Crow a otra clave (Claves → cambiar)
people-not-on = no está en { $server }
people-that-isnt-an-openssh-public = eso no es una clave pública OpenSSH (ssh-ed25519 AAAA… comentario)

## terminal-session
terminal-session-session-ended = sesión terminada
terminal-session-session-ended-exit-status = sesión terminada (código de salida { $code })
terminal-session-session-ended-killed-by-a = sesión terminada (matada por una señal)

## geoip
geoip-the-geoip-database-couldnt-be = no se pudo leer la base de datos GeoIP: { $e }
geoip-the-geoip-database-has-no = la base de datos GeoIP no tiene datos de países
geoip-no-data-directory = no hay carpeta de datos
geoip-bad-database-path = ruta de base de datos incorrecta
geoip-the-geoip-database-couldnt-be-2 = no se pudo descargar la base de datos GeoIP ({ $last })

## appearance
appearance-isnt-a-picture-crow-can = { $source } no es una imagen que Crow pueda leer: { $e }
appearance-cant-create = no se puede crear { $dir }: { $e }
appearance-cant-save = no se puede guardar { $dest }: { $e }
appearance-cant-read = no se puede leer { $stored }: { $e }
appearance-cant-save-2 = no se puede guardar { $cached }: { $e }

## vault
vault-the-vault-is-locked = la bóveda está bloqueada
vault-still-reading-the-os-keyring = todavía leyendo el llavero del sistema; inténtalo de nuevo en un momento
vault-no-os-keyring-is-available = no hay llavero del sistema que guarde la clave de cifrado ({ $why }); activa la contraseña de la bóveda en Bóveda y seguridad
vault-db-lock-poisoned = bloqueo de la base de datos envenenado

## security-hardening
security-hardening-isnt-a-setting-crow-applies = ✕ { $value } no es un ajuste que Crow aplique.
security-hardening-couldnt-read-sshd-config-nothing = ✕ No se pudo leer sshd_config: { $e }. No se cambió nada.
security-hardening-couldnt-write-nothing-was-changed = ✕ No se pudo escribir { $root_dropin }: { $e }. No se cambió nada.

## security-sshd-passwords
security-sshd-passwords-a-fresh-key-login-doesnt-2 = ✕ No funciona un inicio de sesión nuevo con clave ({ $e }). No se cambió nada: desactivar ahora las contraseñas podría dejarte fuera.
security-sshd-passwords-couldnt-read-sshds-settings-nothing = ✕ No se pudieron leer los ajustes de sshd: { $e }. No se cambió nada.
security-sshd-passwords-couldnt-create-etc-ssh-sshd-2 = ✕ No se pudo crear /etc/ssh/sshd_config.d: { $e }. No se cambió nada.
security-sshd-passwords-couldnt-write-nothing-was-changed = ✕ No se pudo escribir { $dropin }: { $e }. No se cambió nada.

## host-diagnose
host-diagnose-sshd-may-be-stopped-or = Puede que sshd esté detenido, o escuchando en otro puerto.

## security-hardening
security-hardening-sshd-accepts-passwords-passwordauthentication-yes = sshd acepta contraseñas (PasswordAuthentication yes)

## lab-multipass
lab-multipass-multipass-launch-didnt-finish-in = multipass launch no terminó en { $launch } minutos
lab-multipass-multipass-launch-failed = multipass launch falló ({ $status })
lab-multipass-multipass-failed = multipass falló

## dr
dr-last-backup-is-old-more = la última copia { $name } tiene { $age }: más que el RPO de { $rpo_minutes } min
dr-last-backup-is-old = la última copia { $name } tiene { $age }

## certs
certs-certbot-is-installed-but-nothing = certbot está instalado pero nada lo ejecuta (ni temporizador ni cron): los certificados no se renovarán solos
certs-acme-sh-is-installed-but = acme.sh está instalado pero falta su tarea de cron: los certificados no se renovarán solos

## team
team-env = entorno { $e }

## Settings → Personalisation → Language
app-language-couldnt-load = No se pudo cargar el idioma { $code }, así que Crow está en inglés: { $e }
app-language-couldnt-create = No se pudo crear { $path }: { $e }
app-language-template-written = Plantilla en inglés guardada en { $path }. Cámbiale el nombre al código de tu idioma (es.ftl, ja.ftl), traduce el texto tras cada = y luego RECARGAR.
app-language-template-not-written = No se guardó la plantilla: { $e }
settings-language-title = IDIOMA
settings-language-about = El idioma de las pantallas de Crow. Los idiomas son archivos .ftl que haces o descargas, guardados en { $folder }. Cada uno se comprueba contra el inglés: lo que le falte o tenga mal se muestra en inglés.
settings-language-open-folder = ABRIR LA CARPETA DE IDIOMAS
settings-language-export-template = EXPORTAR PLANTILLA EN INGLÉS
settings-language-reload = RECARGAR
settings-language-complete = completo
settings-language-coverage = { $percent }% · faltan { $missing }
settings-language-by = de { $author }
settings-language-refused = { $count ->
        [one] Se rechazó { $count } mensaje
       *[other] Se rechazaron { $count } mensajes
    } ({ $errors } errores de sintaxis); { $count ->
        [one] se muestra
       *[other] se muestran
    } en inglés:
settings-language-more = …y { $count } más

## Converted by hand
fleet-rollouts-as-root-sudo = como root (mediante sudo -n)
users-new-user-modal-executes-under-sudo = Ejecuta useradd y usermod -aG con sudo
fleet-overview-host-count = · { $count ->
        [one] { $count } host
       *[other] { $count } hosts
    }
settings-clankers-all-time = { $count } en total
logs-explain-lines = EXPLICAR { $count } { $count ->
        [one] LÍNEA
       *[other] LÍNEAS
    }

app-configs-value-placeholder = valor
fleet-alert-lines-down-now = { $reason } · avisa si sigue caído en la próxima comprobación
fleet-alert-lines-now = ahora
fleet-alert-lines-resolved = resuelta · { $detail } (duró { $lasted })
config-crawler-could-not-read-from = No se pudo leer { $name } de { $host }: { $e }
config-crawler-could-not-read = No se pudo leer { $name } de { $host }
config-pending-diff-rail-clean = limpio
app-overview-search-journal = busca en el journal (regex; en minúsculas no distingue mayúsculas)…  ( / )

## Danger Zone: what is about to happen (English shows beside a translation)
danger-zone-what-poweroff = Apagar el servidor
danger-zone-what-reboot = Reiniciar el servidor
danger-zone-what-boot = Arrancar el servidor
danger-zone-what-snapshot = Hacer una instantánea del servidor
danger-zone-what-flush = Eliminar todas las reglas del cortafuegos y desactivarlo
danger-zone-what-kill = Detener todos los contenedores de laboratorio que creó Crow
danger-zone-what-other = Ejecutar esta acción

## Settings rows
setting-general-refresh-interval-label = Intervalo de actualización
setting-general-refresh-interval-help = Cada cuántos segundos se consultan en segundo plano los servicios y la telemetría.
setting-connection-control-master-label = Conexión maestra
setting-connection-control-master-help = Reutiliza una conexión TCP por host para todos los paneles. Reduce los apretones de manos de ~14 a 1.
setting-connection-keepalive-interval-label = Intervalo de keepalive
setting-connection-keepalive-interval-help = Segundos entre sondeos keepalive. Más bajo detecta antes los hosts caídos, a costa de más tráfico.
setting-connection-connect-timeout-label = Tiempo de conexión
setting-connection-connect-timeout-help = Abandona un apretón de manos TCP pasado este tiempo.
setting-security-auto-lock-minutes-label = Minutos de bloqueo automático
setting-security-auto-lock-minutes-help = Bloquea la bóveda y borra de la RAM las claves descifradas tras un tiempo de inactividad.
setting-appearance-fleet-background-label = Fondo de la flota
setting-appearance-fleet-background-help = Imagen detrás de la lista de servidores de la Flota (se guarda una copia en ~/.config/crow/backgrounds). Vacío para ninguna.
setting-appearance-fleet-background-opacity-label = Opacidad del fondo de la flota
setting-appearance-fleet-background-opacity-help = Cuánto se ve la imagen. Baja deja la tabla fácil de leer.
setting-appearance-fleet-background-opacity-option-5 = 5 % · un toque
setting-appearance-fleet-background-opacity-option-10 = 10 % · sutil
setting-appearance-fleet-background-opacity-option-15 = 15 % · visible
setting-appearance-fleet-background-opacity-option-25 = 25 % · intensa
setting-appearance-fleet-background-opacity-option-35 = 35 % · marcada
setting-appearance-fleet-background-blur-label = Desenfoque del fondo de la flota
setting-appearance-fleet-background-blur-help = Suaviza la imagen para que las filas se lean bien encima.
setting-appearance-fleet-background-blur-option-0 = Ninguno
setting-appearance-fleet-background-blur-option-4 = Ligero
setting-appearance-fleet-background-blur-option-8 = Suave
setting-appearance-fleet-background-blur-option-16 = Fuerte
setting-appearance-fleet-background-blur-option-32 = Esmerilado
setting-servers-archive-purge-days-label = Días hasta eliminar archivados
setting-servers-archive-purge-days-help = Cuánto tiempo conserva sus datos un servidor archivado antes de que Crow los elimine. El registro del servidor nunca se elimina.
setting-servers-archive-purge-days-option-30 = 30 días
setting-servers-archive-purge-days-option-60 = 60 días
setting-servers-archive-purge-days-option-90 = 90 días · por defecto
setting-servers-archive-purge-days-option-180 = 180 días
setting-servers-archive-purge-days-option-never = No eliminar nunca
setting-appearance-terminal-font-size-label = Tamaño de letra de la terminal
setting-appearance-terminal-font-size-help = Tamaño de letra en las terminales de servidor, en puntos.
setting-appearance-start-maximized-label = Abrir maximizado
setting-appearance-start-maximized-help = Abre Crow maximizado. Se aplica la próxima vez que se inicie Crow.
setting-appearance-language-label = Idioma
setting-appearance-language-help = El idioma de las pantallas de Crow: en, o el código de un paquete de la carpeta de idiomas (es para es.ftl).
setting-servers-host-key-max-age-days-label = Antigüedad máxima de la clave de host (días)
setting-servers-host-key-max-age-days-help = Hasta qué antigüedad puede llegar la clave de host SSH de un servidor antes de que ALERTAS DE LA FLOTA avise de que toca rotarla (Flota → Zona de peligro → Rotar todas las claves de host).
setting-servers-host-key-max-age-days-option-180 = 180 días
setting-servers-host-key-max-age-days-option-365 = 1 año · por defecto
setting-servers-host-key-max-age-days-option-730 = 2 años
setting-servers-host-key-max-age-days-option-never = No avisar nunca
setting-servers-geoip-regions-label = Regiones por GeoIP
setting-servers-geoip-regions-help = Averigua el país de los servidores sin metadatos de nube a partir de su IP pública, consultándolo en este equipo en la base de datos gratuita de países de DB-IP (descargada de db-ip.com más o menos una vez al mes; nunca se envía la dirección de ningún servidor). Geolocalización IP de DB-IP (db-ip.com), CC BY 4.0.
setting-general-update-check-label = Buscar actualizaciones
setting-general-update-check-help = Consulta GitHub una vez al día por si hay un Crow más nuevo y lo indica junto a la versión. Solo se pide la lista de versiones; no se envía nada sobre ti ni tus servidores.
setting-general-update-prereleases-label = Versiones preliminares
setting-general-update-prereleases-help = Ofrecer también versiones preliminares (betas). Las compilaciones que ya son preliminares siempre ven las más nuevas.
setting-general-update-auto-install-label = Instalar actualizaciones solo
setting-general-update-auto-install-help = Descarga e instala las versiones nuevas por sí solo, únicamente después de que slsa-verifier confirme la procedencia firmada de la versión (debe estar instalado). Crow ofrece reiniciar y nunca se reinicia solo.

## Plugins
plugin-multipass-name = Multipass
plugin-multipass-about = VMs de Ubuntu en este equipo: lanza una con la clave de Crow y se une a la flota; impórtalas, enciéndelas y elimínalas.
plugin-podman-name = Podman
plugin-podman-about = Contenedores de laboratorio en este equipo, añadidos como nodos de pruebas.
plugin-distrobox-name = Distrobox
plugin-distrobox-about = Entornos de Distrobox (con Podman o Docker debajo), como contenedores de laboratorio.
plugin-docker-name = Docker
plugin-docker-about = Contenedores de laboratorio con Docker.
plugin-containers-name = Contenedores en servidores
plugin-containers-about = Docker y Podman en tus servidores, por SSH: contenedores, pilas compose, puertos publicados, registros, iniciar/detener/reiniciar y la varita. Añade Contenedores a la barra lateral de cada servidor.
plugin-linode-name = Linode
plugin-linode-about = Tus instancias de Linode: impórtalas, lee sus regiones, enciéndelas y apágalas y hazles instantáneas.
plugin-upcloud-name = UpCloud
plugin-upcloud-about = Tus servidores de UpCloud: impórtalos, lee sus zonas, enciéndelos y apágalos y hazles instantáneas.
plugin-email-name = Email
plugin-email-about = Alertas por correo a través de tu propio servidor SMTP: las críticas al momento, los avisos en un resumen. Se envían mientras Crow está en marcha.


## Product names (shown as they are)
lab-distrobox = Distrobox
lab-podman = Podman
lab-multipass = Multipass
lab-docker = Docker
containers-docker = Docker
containers-podman = Podman

## Tests
i18n-test-args = { $server }: { $count ->
    [one] { $count } servidor
   *[other] { $count } servidores
}
i18n-test-plain = Hola

## Audit log: what a fleet run's records mean
audit-model-offboarded-on = baja en { $target }
audit-model-key-revoked-on = clave revocada en { $target }
audit-model-account-set-up-on = cuenta creada en { $target }
audit-model-certificate-renewed-on = certificado renovado en { $target }
audit-model-hardening-fix-on = corrección de endurecimiento en { $target }
audit-model-risk-accepted-on = riesgo aceptado en { $target }
audit-model-command-run-on = comando ejecutado en { $target }
audit-model-config-rolled-out-to = configuración desplegada en { $target }
audit-model-rollout-undone-on = despliegue revertido en { $target }
audit-model-updates-applied-on = actualizaciones aplicadas en { $target }
audit-model-host-keys-rotated-on = claves de host rotadas en { $target }
audit-model-sessions-ended-on = sesiones SSH cerradas en { $target }
audit-model-brought-back-on = devuelto a la línea base en { $target }
audit-model-permissions-changed = permisos cambiados: { $target }
audit-model-plugin-accepted = plugin { $target } aceptado
audit-model-plugin-refused = plugin { $target } rechazado
audit-model-plugin-revoked = plugin { $target } ya no aceptado
audit-model-plugin-ran = { $target }: se ejecutó { $action }
identity-bar-kernel = kernel
identity-bar-role = rol

## Audit log filters
audit-all = todo
audit-kind-config = config.
audit-kind-service = servicio
audit-kind-process = proceso
audit-kind-firewall = cortafuegos
audit-kind-user = usuario
audit-kind-power = energía
audit-kind-ssh = ssh
audit-kind-enroll = alta
audit-kind-crow = crow
