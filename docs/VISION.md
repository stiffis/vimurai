# Vimurai: el dojo de memoria muscular

Vimurai no es una chuleta de comandos con una interfaz bonita. Es un gimnasio
de práctica deliberada: plantea una tarea concreta sobre código realista,
captura cómo se resolvió y vuelve a presentar cada habilidad justo cuando
conviene repasarla.

Esta visión consolida la investigación y los borradores originales que viven
en `src_old/local/`. Es la fuente breve de verdad para la implementación
actual.

## Los cuatro pilares

1. **Práctica auténtica.** Un mini editor Vim funcional permite aprender en el
   mismo ciclo de modo, movimiento y edición que se transfiere a Vim/Neovim.
2. **Repetición espaciada.** El Daily Drill crea una cola finita de 3, 5 o 10
   minutos. SM-2 programa cada concepto a partir de aciertos, errores, pistas y
   eficiencia; un fallo también es información útil.
3. **Gamificación ligera.** XP, cinturones, estrellas, racha, actividad y logros
   sirven de brújula. Nunca convierten el tiempo en una barrera para aprender.
4. **Progresión con andamiaje.** Cada cinturón introduce pocas ideas y las
   combina después: movimiento, precisión, gramática, objetos de texto,
  navegación estructural y reutilización.

## Bucle del alumno

```text
objetivo realista → intento en mini-Vim → feedback inmediato
        ↑                                  ↓
 repaso programado ← resultado semántico + reflexión
```

Se miden acciones semánticas, no caracteres escritos. `12j` es una acción; una
búsqueda completa también. Una ruta válida pero poco eficiente aprueba y luego
invita a optimizarse.

## Modos del producto

- **Daily Drill:** revisión personalizada y acotada; primero lo vencido, luego
  conceptos nuevos desbloqueados y finalmente repaso mixto.
- **Academia:** campaña por cinturones, sin presión de tiempo y con objetivos,
  contexto, pista progresiva y solución de referencia.
- **Sandbox:** editor libre con snippets de Rust, Python y texto/logs; no afecta
  negativamente al progreso.
- **Progreso:** nivel, XP, precisión, racha, heatmap, dominio y logros.
- **Referencia y ajustes:** catálogo buscable y configuración local.

## Cinturones

| Cinturón | Identidad | Núcleo |
|---|---|---|
| Blanco | Survivor | `hjkl`, palabras e inserción segura |
| Amarillo | Sniper | `fFtT`, bordes de línea y precisión |
| Naranja | Refactorer | conteos y gramática operador + motion |
| Rojo | Surgeon | Visual y objetos de texto |
| Azul | Architect | búsqueda, estructura y saltos largos |
| Negro | Wizard | registros, pegado y repetición de búsqueda |

## Reglas de experiencia

- Las teclas imprimibles pertenecen al editor durante una práctica. Ayuda,
  reinicio y salida usan teclas de función o combinaciones con Ctrl.
- Kage, el gato-sensei, reacciona al estado pedagógico; no tiene atajos que
  compitan con motions de Vim.
- El buffer siempre conserva la mayor parte del espacio. La interfaz reduce u
  oculta paneles antes de sacrificar el área de práctica.
- La interfaz hereda el fondo real del emulador y elige Gruvbox Dark o Light a
  partir de su luminancia; Vimurai nunca impone un lienzo opaco.
- Una terminal demasiado pequeña muestra una pantalla segura en vez de hacer
  cálculos de layout que puedan fallar.
- Coordenadas de texto usan caracteres Unicode coherentemente; nunca se
  mezclan offsets de bytes con columnas del cursor.
- Todo el progreso es local. SQLite usa transacciones y los tests trabajan con
  bases en memoria, nunca con el perfil real.

## Arquitectura

```text
entrada terminal
   ├─ navegación de aplicación → rutas / overlays
   └─ práctica activa → parser Vim → acción tipada → editor
                                             ├─ feedback de Kage
                                             ├─ evaluación del ejercicio
                                             └─ SM-2 + SQLite

estado inmutable para render → componentes Ratatui responsivos
```

El editor no depende de Ratatui. El currículo es declarativo y validable. La
persistencia se puede inyectar en memoria. La sesión de terminal usa RAII para
restaurar raw mode, alternate screen, cursor y bracketed paste incluso ante un
panic.

## Principio de alcance

La versión local prioriza profundidad y corrección en el flujo esencial. Las
integraciones sociales, nube, IA y plugins siguen siendo posibles extensiones,
pero no justifican degradar el mini-Vim, el currículo o la privacidad.
