# Plan Maestro Acordado: MMORPG Estático estilo World of Warcraft sobre Veloren

Este plan consolida todas las decisiones tomadas y el estado de implementación para convertir Veloren en un **MMORPG masivo, temático, estructurado y persistente**.

---

## 1. Pilares de Diseño Acordados

| Pilar | Decisión de Diseño | Estado |
| :--- | :--- | :--- |
| **Geografía y Mundo** | **Mundo Estático y Permanente**: Se congela el mapa oficial horneado de Veloren (`assets/world/map/veloren_0_18_0_0.bin`) como el continente base fijo ($1024 \times 1024$ chunks $\approx 1073\text{ km}^2$). Toda la geografía, montañas, ríos y costas quedan establecidos de forma permanente. | **Implementado** |
| **Aparición Segura** | **Spawn Plano Garantizado**: Algoritmo `find_flat_accessible_pos` que verifica la varianza de altura para evitar que los jugadores aparezcan en techos, montañas escarpadas o árboles. | **Implementado** |
| **Facciones y Conflicto** | **Dos Facciones Rivales (Alianza vs Horda)**: Cada facción cuenta con su propia capital, zonas iniciales seguras (Lvl 1–10) y zonas intermedias/avanzadas disputadas con PvP abierto activado. Fuego amigo entre aliados deshabilitado en combate. | **Implementado** |
| **Progresión de Niveles** | **Sistema de Nivel Global (1 al 60)**: Las entidades y jugadores muestran su nivel `[Nvl X] Nombre` con color de dificultad WoW. La barra de XP arranca en Nivel 1. | **Implementado** |
| **Clases y Roles (Trinidad)** | **Clases Fijas y Roles**: Guerrero (Tanque), Paladín (Tanque/Soporte), Sacerdote (Sanador), Mago (DPS), Cazador (DPS), Pícaro (DPS), con armas iniciales temáticas (escudo, cetro, dagas dobles). | **Implementado** |
| **Sistema de Amenaza (Threat/Aggro)** | **Prioridad por Rol de Combate**: Los tanques generan 2.5x más aggro en la IA (`is_more_dangerous_than_target`), los DPS 1.0x y los sanadores 0.5x para mantener la mecánica clásica de mazmorras. | **Implementado** |
| **Sistema de Misiones (*Quests*)** | **Motor de Misiones y Marcadores Overhead**: Marcadores visuales 3D pulsantes `!` y `?` sobre los NPCs interactuables, con ventana de diálogo y recompensas en monedas y XP. | **Implementado** |
| **Mazmorras y Raids** | **Escalado de Contenido Grupal**: Metadatos de mazmorras por rango de nivel (ej. Cavernas Gnarling 5-12, Minas Enanas 32-42, etc.) y Castillo Vampírico como Raid de nivel 60 para 10 jugadores. | **Implementado** |
| **Economía y Comercio** | **Economía Clásica WoW**: $100\text{ Cobre} = 1\text{ Plata}$, $100\text{ Plata} = 1\text{ Oro}$ ($10,000\text{ Cobre} = 1\text{ Oro}$) con formateo numérico (`5g 20s 15c`) y descriptivo (`5 oro, 20 plata, 15 cobre`). | **Implementado** |
| **Densidad de Monstruos** | **Distribución estilo WoW**: Densidad base multiplicada por 5 (`BASE_DENSITY`), con manadas de 2 a 4 enemigos agresivos en caminos y llanuras. | **Implementado** |
| **Cementerios de Zona** | **Graveyards Fijos**: Al morir, el personaje reaparece en el cementerio correspondiente a la zona donde cayó (`zone.graveyard_wpos`). | **Implementado** |
| **Notificación Territorial** | **Zone Splash Messages**: Notificación en el HUD al cruzar entre zonas con el nombre de la región, rango de nivel y tipo de territorio (Seguro / Disputado). | **Implementado** |
| **Idioma por Defecto** | **Español Latino (`es-419`)**: Configuración regional por defecto y traducciones completas en archivos Fluent (`.ftl`). | **Implementado** |

---

## 2. Estructura de Facciones y Territorios

### 🛡️ Alianza
- **Especies**: Humanos, Enanos, Elfos
- **Capital**: *Ciudadela de la Luz* (Coordenadas $X: 14000, Y: 14000$)
- **Zona de Inicio**: *Valle Dorado* (Niveles 1–10, Territorio Seguro)
- **Cementerio**: $X: 14020, Y: 14010$

### ⚔️ Horda
- **Especies**: Orcos, Draugr, Danari
- **Capital**: *Bastión de Ogron* (Coordenadas $X: 18000, Y: 18000$)
- **Zona de Inicio**: *Estepas Áridas* (Niveles 1–10, Territorio Seguro)
- **Cementerio**: $X: 18020, Y: 18010$

### ⚔️ Zonas en Disputa (PvP Abierto)
- **Región**: *Tierras Disputadas* (Niveles 10–25)
- **Centro**: $X: 16000, Y: 16000$ (Radio 8000 bloques)
- **Regla**: Combate abierto automático entre jugadores de facciones opuestas.

---

## 3. Mazmorras e Incursiones del Continente

1. **Cavernas Gnarling**: Niveles 5–12 (Mazmorra inicial, 5 jugadores)
2. **Templo Helado de Adlet**: Niveles 12–20 (5 jugadores)
3. **Catacumbas Ancestrales Haniwa**: Niveles 18–28 (5 jugadores)
4. **Santuario de las Profundidades**: Niveles 20–30 (5 jugadores)
5. **Cuevas Acuáticas Sahagin**: Niveles 25–35 (5 jugadores)
6. **Minas Olvidadas de los Enanos**: Niveles 32–42 (5 jugadores)
7. **Mausoleo de Terracota**: Niveles 38–48 (5 jugadores)
8. **Fortaleza Colmena Myrmidon**: Niveles 45–54 (5 jugadores)
9. **Bastión del Vacío Cultista**: Niveles 50–58 (5 jugadores)
10. **Castillo Sanguíneo del Conde**: **Incursión de Banda / Raid Nivel 60 (10 jugadores)**

---

## 4. Archivos Modificados y Creados

- **`common/src/zone.rs`**: Zonas del mundo, facciones, coordenadas y búsqueda de zona por posición.
- **`common/src/class.rs`**: Clases, roles de combate (Tanque, Healer, DPS), multiplicadores de amenaza y vida.
- **`common/src/quest.rs`**: Estructuras y componentes para el motor de misiones.
- **`common/src/economy.rs`**: Conversión de Cobre/Plata/Oro y formateo WoW.
- **`common/src/terrain/site.rs`**: Metadatos de mazmorras, rangos de nivel y tamaños de grupo/raid.
- **`common/src/comp/player.rs`**: Campo `faction` y reglas de PvP entre facciones (`Player::may_harm`).
- **`server/src/events/entity_manipulation.rs`**: Reaparición de personajes en el cementerio de su zona.
- **`server/src/events/entity_creation.rs`**: Asignación automática de facción al cargar el personaje.
- **`server/agent/src/action_nodes.rs`**: IA de combate ponderada por nivel de amenaza de clase (Tank 2.5x aggro).
- **`voxygen/src/session/mod.rs`**: Detección y notificación de cambio de zona en el HUD.
- **`voxygen/src/hud/overhead.rs`**: Marcadores 3D `!` y `?` sobre NPCs y formato de nombre con nivel `[Nvl X]`.
- **`voxygen/src/hud/skillbar.rs`**: Barra de experiencia iniciando en Nivel 1.
- **`voxygen/src/settings/language.rs`**: Idioma por defecto en `es-419`.
- **`assets/voxygen/i18n/es-419/`**: Localización completa de tutoriales, misiones y habilidades.
- **`assets/world/wildlife/spawn/temperate/plains.ron`**: Manadas de 2 a 4 enemigos agresivos.
- **`server/src/settings/mod.rs`**: Persistencia de terreno experimental activada.
