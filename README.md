# DigiSurf: A TUI signal waveform viewer

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/SeanMcLoughlin/digisurf/actions/workflows/ci.yml/badge.svg)](https://github.com/SeanMcLoughlin/digisurf/actions/workflows/ci.yml)

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
- [x] Fuzzy search for signal names
- [-] FSDB parsing
  - FSDB is a proprietary format by Synopsys and you cannot use their FFT C API unless you have a license agreement with them, which I do not.
- [l] Wave DB file streaming for performance
  - Very low priority
- [x] Command mode
  - [ ] Tab complete
  - [x] Up arrow for previous commands
  - [x] Left/Right arrows to move cursor to different positions
  - [ ] Ctrl-U clear backwards from cursor
  - [ ] Ctrl-W clears word before cursor
  - [ ] Ctrl-C exit
- [ ] Color picker for each wave+signal, marker, etc.
- [-] Wave style for each wave
  - Doesn't look good.
- [-] Embedded text editor, showing signal values at marker
  - Requires some simulator output which I don't have.
- [ ] Export signal selections and markers to config file of some sort
- [ ] Export selection to SVG(?)
