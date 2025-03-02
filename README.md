# DigiSurf

A TUI signal waveform viewer.

## To Do

- [ ] Wrap the waveform in a chart and use the chart axes to show a scalable timescale view.
- [x] Non-binary wave support (e.g. show values in hexadecimal instead of binary high/low)
- [ ] Expand bus values in a tree structure in the signal list/waveform viewer
- [x] X and Z support
- [ ] Simple mouse events for clicking signal names to highlight them.
- [ ] Markers
  - [x] Markers on click events
  - [ ] Saveable markers based on marker location of click event
- [x] Zoom into selection of waves based on mouse click-and-drag selection
- [ ] Fuzzy search for signal names
- [-] FSDB parsing
  - FSDB is a proprietary format by Synopsys and you cannot use their FFT C API unless you have a license agreement with them, which I do not.
- [ ] Wave DB file streaming for performance
- [x] Vim-like command mode (or at least better keybindings, depends on how complex this tool will get)
- [ ] Color picker for each wave+signal, marker, etc.
- [ ] Wave style for each wave
- [ ] Embedded text editor, showing signal values at marker
- [ ] Export signal selections and markers to config file of some sort
- [ ] Export selection to SVG(?)
