# thumbcraft

**Document thumbnail generator for Node.js and TypeScript.** Turn a Word document, an Excel spreadsheet or a PowerPoint presentation into a preview image (PNG, JPEG or WebP) in about a tenth of a second, with no LibreOffice, no headless browser, no GPU and no cloud service.

thumbcraft is a thin TypeScript wrapper around a fast native binary written in Rust. It is built for file-upload pipelines: render the first page of a `.docx`, the first sheet of an `.xlsx` or the first slide of a `.pptx` into a thumbnail, from a file on disk or straight from a `Buffer` in memory.

> **Status: pre-release.** The package is not published on npm yet. Until it is, build the binary from the [repository](https://github.com/klawdyo/thumbcraft) and point the `THUMBCRAFT_BIN` environment variable at it (see [Finding the binary](#finding-the-binary)).

## Features

- **DOCX, XLSX and PPTX to image**: Word (`.docx`, `.doc`, `.odt`, `.rtf`, `.html`, `.md`, `.txt` and more), spreadsheets (`.xlsx`, `.xls`, `.ods`, `.csv`, `.tsv`) and presentations (`.pptx`, `.ppsx`, `.potx`) with one API.
- **Document preview generator** for upload flows (Express, Fastify, NestJS, Multer): `Buffer` in, image out.
- **No LibreOffice or headless Chrome**: a self-contained native binary, so it runs in a slim Docker image or a serverless-style environment without installing an office suite.
- **Resize, crop and fit built in**: width and height, `cover` or `contain`, gravity, background color, automatic margin trimming (`trim`) and a fractional crop rectangle. No `sharp` needed (it combines well with it if you already use it).
- **PNG, JPEG and WebP output** with adjustable JPEG quality.
- **Any page or slide**: pick the page, slide or print page to render.
- **Typed errors**: every failure mode is a distinct TypeScript error class.
- **Safe by design**: one short-lived process per file with a timeout, so a bad document cannot take your Node.js process down.
- **Zero runtime dependencies**, dual ESM and CommonJS (`import` and `require`), full TypeScript types.

## Install

```sh
npm install thumbcraft
```

Requires Node.js 20 or newer. Supported platforms (planned): Windows x64 and Linux x64 (glibc). macOS, ARM and Alpine (musl) are not supported.

## Quick start

Generate a thumbnail from a file:

```ts
import { Thumbcraft } from 'thumbcraft';

await Thumbcraft.fromFile('./report.docx').toFile('./report.png', {
  width: 320,
  height: 240,
  fit: 'cover',
});
```

Generate a preview image from an uploaded file in memory (for example from Multer) and get the PNG bytes back:

```ts
import { Thumbcraft } from 'thumbcraft';

const png: Buffer = await Thumbcraft
  .fromBuffer(upload.buffer, upload.originalname) // the name's extension helps pick the engine
  .toBuffer({ width: 300, height: 300, fit: 'contain', trim: true });
```

Render a specific slide as WebP:

```ts
const webp = await Thumbcraft
  .fromFile('./deck.pptx')
  .toBuffer({ page: 3, width: 800, format: 'webp' });
```

## API

### `Thumbcraft.fromFile(path, config?)`

Creates a job from a document on disk. Returns a `ThumbcraftJob`.

### `Thumbcraft.fromBuffer(buffer, filename, config?)`

Creates a job from bytes in memory. `filename` is the original file name: its extension helps thumbcraft pick the right engine (Word, spreadsheet or presentation). Returns a `ThumbcraftJob`.

### `job.toBuffer(options?)`

Renders the document and resolves with the image as a `Buffer`.

### `job.toFile(outputPath, options?)`

Renders the document and writes the image to `outputPath`. The extension chooses the format (`.png`, `.jpg`, `.webp`) unless you pass `options.format`.

Every call starts a new `thumbcraft` process and never reuses it.

### `RenderOptions`

All options are optional; the defaults are the binary's defaults.

| Option | Type | Meaning |
|---|---|---|
| `width`, `height` | `number` | Final size in pixels. One alone keeps the aspect ratio. |
| `fit` | `'cover' \| 'contain'` | With both sizes: fill the box and crop (default) or fit the whole page inside it. |
| `gravity` | `'top' \| 'bottom' \| 'left' \| 'right' \| 'center' \| 'top-left' \| 'top-right' \| 'bottom-left' \| 'bottom-right'` | Anchor for cropping or placement. Default: `top` with `cover`, `center` with `contain`. |
| `background` | `string` | Background for `contain` and under transparency: `'white'` (default), `'transparent'` or `'#RRGGBB'`. |
| `trim` | `boolean` | Remove blank or transparent margins around the content. |
| `trimPad` | `number` | Margin kept by `trim`, in pixels of the native render (default 8). |
| `crop` | `{ x, y, width, height }` | Crop rectangle as fractions (0 to 1) of the rendered page, applied before resizing. |
| `page` | `number` | Page, slide or print page to render, starting at 1 (default 1). |
| `scale` | `number \| 'auto'` | Native render scale in pixels per point. `'auto'` (default) renders at about twice the target size for sharp results. |
| `filter` | `'lanczos' \| 'area'` | Downscale filter (default `lanczos`). |
| `format` | `'png' \| 'jpeg' \| 'webp'` | Output format. WebP is lossless. |
| `quality` | `number` | JPEG quality, 1 to 100 (default 85). |

### Configuration

The second argument of `fromFile` and `fromBuffer` accepts:

| Field | Meaning |
|---|---|
| `timeoutMs` | Time limit per render in milliseconds (default 30000). The process is killed when it is exceeded. |
| `binaryResolver` | Custom way to find the binary (see below). |
| `runner`, `fileSystem` | Replace process execution and temp-file access, useful for unit tests. |

## Error handling

All errors extend `ThumbcraftError`, which carries `exitCode` (when the binary produced one) and the raw `stderr`. The messages of the binary are in Portuguese.

| Error class | When |
|---|---|
| `UsageError` | An option is invalid or out of range. |
| `UnsupportedFormatError` | The format is not supported (for example `.ppt`, `.odp`, PDF, unknown extension). |
| `InvalidFileError` | The file is corrupt, empty, password-protected, or the requested page does not exist. |
| `InternalError` | Internal failure, including failing to write the output. |
| `ProcessCrashError` | The process died unexpectedly (an abnormal exit status or a signal). |
| `TimeoutError` | The render exceeded `timeoutMs`. |
| `BinaryNotFoundError` | The `thumbcraft` binary could not be found or started. |

```ts
import { Thumbcraft, UnsupportedFormatError, InvalidFileError } from 'thumbcraft';

try {
  await Thumbcraft.fromFile('./slides.ppt').toFile('./slides.png');
} catch (error) {
  if (error instanceof UnsupportedFormatError) {
    // legacy .ppt is not supported: skip the preview or ask for a .pptx
  } else if (error instanceof InvalidFileError) {
    // damaged document: reject the upload
  } else {
    throw error;
  }
}
```

## Finding the binary

The wrapper runs a native `thumbcraft` executable and looks for it in this order:

1. The `THUMBCRAFT_BIN` environment variable, if set. Useful in development or to use a custom build.
2. The prebuilt platform package that npm installed next to `thumbcraft` as an optional dependency, the same way `sharp` does it: `@thumbcraft/win32-x64-msvc` on Windows x64 or `@thumbcraft/linux-x64-gnu` on Linux x64 (glibc).

```sh
THUMBCRAFT_BIN=/path/to/thumbcraft node app.js   # optional override
```

If you develop on Windows and run in a Linux container, tell npm to install both binaries by adding `supportedArchitectures` to your own `package.json` (npm 10 or newer):

```json
{
  "supportedArchitectures": { "os": ["win32", "linux"], "cpu": ["x64"], "libc": ["glibc"] }
}
```

If no binary is found, the error names the platform and the package that is missing. Do not install with `--omit=optional`, or the platform package is skipped.

## How it works

thumbcraft bundles three Rust rendering engines (WordCraft, GridCraft and DeckCraft from the [storytold](https://github.com/storytold) organization) into one binary. The engine is chosen from the file content, and the page is rendered at high resolution and then cropped and scaled down, which keeps text crisp in small thumbnails. A render usually takes well under 150 ms on a regular machine for typical documents.

Because a native engine failure aborts its process by design, the wrapper starts one process per render and enforces a timeout, so one bad file never affects your server.

## Known limitations

- PDF input is not supported yet.
- Legacy `.ppt` and `.odp` presentations are not supported.
- Spreadsheets render the first print page (A4, no zoom), so wide sheets show only their top-left corner; use `trim` to focus on the content.
- Fonts missing on the machine are replaced by similar ones, so line breaks can differ slightly from Word.

## Links

- Source, binary builds and documentation: https://github.com/klawdyo/thumbcraft
- License: Apache-2.0

*Keywords: thumbnail generator, document preview, docx to png, xlsx to png, pptx to png, office document thumbnail, Word, Excel, PowerPoint, OpenDocument, RTF, CSV preview, file upload preview, image from document, Node.js, TypeScript, Rust, native binary, no LibreOffice.*
