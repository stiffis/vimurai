# Vimurai - Design Specification v1.0

Este documento detalla la funcionalidad, experiencia de usuario (UX) y requerimientos técnicos para cada módulo de la aplicación Vimurai.

## 1. Daily Drill (El Gimnasio Diario)
**Objetivo:** Mantenimiento de memoria muscular eficiente mediante Repetición Espaciada (SM-2).

### Funcionalidad
- **Lógica de Selección:**
    1.  Cargar ejercicios "Due" (Vencidos) de la DB.
    2.  Si hay < 5 minutos de contenido estimado, rellenar con ejercicios nuevos (New) del nivel actual desbloqueado.
    3.  Si no hay nada pendiente ni nuevo, ofrecer "Repaso General" (ejercicios aleatorios).
- **Flujo de Sesión:**
    - Mostrar ejercicio -> Usuario resuelve -> Feedback (Éxito/Fallo) -> Siguiente.
    - **Session Summary:** Al terminar la cola de ejercicios, mostrar un resumen: "Ejercicios: 15 | Aciertos: 90% | XP Ganada: 150".

### UI/UX
- **Pantalla de Juego:** La misma que ya tenemos (Gutter, Status Bar).
- **Indicador de Racha:** Pequeño icono de fuego 🔥 con los días seguidos en la esquina superior derecha.
- **Feedback:** Sonido (opcional) y flash visual verde/rojo.

### Requerimientos Técnicos
- Persistencia de `next_review` en SQLite (Ya implementado).
- Algoritmo de "Session Queue" que no sea infinito (limitar a N ejercicios o X minutos).

---

## 2. Guided Learning (La Academia / Campaña)
**Objetivo:** Aprender conceptos nuevos de forma estructurada y secuencial, sin presión de tiempo.

### Funcionalidad
- **Selector de Nivel:** Una lista jerárquica de Cinturones/Niveles.
    - *Survivor (White Belt)* [====..] 40%
    - *Sniper (Yellow Belt)* [=.....] 10%
- **Modo Estudio:**
    - Al entrar en un nivel, se muestra la lista de ejercicios (R1, R2, R3...).
    - El usuario puede elegir cualquiera, aunque se recomienda el orden.
    - **No afecta negativamente al SM-2:** Los fallos aquí no bajan tu puntuación, pero los aciertos pueden marcar un ejercicio como "Visto" por primera vez.

### UI/UX
- **Layout:** Panel izquierdo con la lista de Niveles. Panel derecho con la descripción del nivel y lista de ejercicios.
- **Estado Visual:**
    - 🔒 Candado: Nivel bloqueado (requiere completar el anterior).
    - ✅ Check: Ejercicio dominado.
    - ⭕ Círculo: Ejercicio disponible.

### Requerimientos Técnicos
- Nueva pantalla `Screen::LevelSelect`.
- Lógica para calcular `%` de completado por nivel consultando `CommandDatabase` + `UserProgressDB`.

---

## 3. Free Practice (El Sandbox)
**Objetivo:** Experimentación libre y calentamiento sin objetivos específicos.

### Funcionalidad
- **Entorno Libre:** Un buffer de Vim funcional sin condiciones de victoria.
- **Carga de Snippets:** Posibilidad de cargar código real para practicar.
    - Presionar `F1`: Cargar Snippet de Rust.
    - Presionar `F2`: Cargar Snippet de Python.
    - Presionar `F3`: Cargar Texto Plano.
- **Reset:** `F5` para limpiar el buffer.

### UI/UX
- **Status Bar:** Debe indicar "FREE MODE - No Score".
- **Visualización de Teclas:** Mostrar en tiempo real las teclas pulsadas en una esquina (Keycaster) para que el usuario sea consciente de sus movimientos.

### Requerimientos Técnicos
- Una colección de `String` constantes con código de ejemplo en `src/data/snippets.rs`.
- Desactivar la lógica de `check_exercise_completion` en este modo.

---

## 4. Progress (Sala de Trofeos)
**Objetivo:** Motivación y visualización de métricas a largo plazo.

### Funcionalidad
- **Estadísticas Clave:**
    - Nivel actual y Barra de XP circular o lineal.
    - Total de comandos aprendidos vs. total disponible.
- **Heatmap de Actividad:** Un gráfico tipo GitHub (cuadraditos verdes) mostrando la actividad de los últimos 30 días.
- **Logros:** Lista de medallas (ej: "Sniper: Usa 'f' 50 veces", "Speedster: Completa 10 ejercicios en < 1 min").

### UI/UX
- Uso intensivo de `ratatui::widgets::Chart` o `Gauge`.
- Layout en Grid: Stats arriba, Heatmap en medio, Logros abajo.

### Requerimientos Técnicos
- Consultas SQL más complejas (`GROUP BY date`) para el heatmap.
- Sistema de "Triggers" de logros (verificar logros al guardar progreso).

---

## 5. Settings (Configuración)
**Objetivo:** Personalización persistente.

### Funcionalidad
- **Opciones:**
    - `Sound Effects`: On/Off.
    - `Difficulty`: Afecta al multiplicador de intervalo del SM-2.
    - `Vim Keymap`: (Futuro) Permitir remapear Escape a `jj` o `jk`.
- **Persistencia:** Guardar cambios en disco.

### UI/UX
- Formulario simple (ya existente, pero necesita guardar).

### Requerimientos Técnicos
- Crear tabla `settings` en SQLite o usar un archivo `config.toml` en `~/.config/vimurai/`. (Recomendado: Tabla KV en SQLite para mantener todo en un solo archivo).

---

## 6. Help (Referencia)
**Objetivo:** Referencia rápida sin salir de la app.

### Funcionalidad
- **Diccionario de Comandos:** Lista generada dinámicamente desde `CommandDatabase`.
- **Buscador:** Poder escribir `/` y filtrar la lista (ej: buscar "delete").

### UI/UX
- Tabla con columnas: `Tecla | Nombre | Descripción`.
- Scrollable.

---

## Resumen de Prioridades de Implementación

1.  **Guided Learning:** Es vital para que el usuario entienda qué está haciendo antes de que el Daily Drill lo evalúe.
2.  **Free Practice (Snippets):** Fácil de hacer y muy útil.
3.  **Progress (Visuals):** Importante para la retención (retention).
4.  **Settings Persistence:** Calidad de vida.
