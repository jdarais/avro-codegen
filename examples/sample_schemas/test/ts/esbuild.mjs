import { build } from "esbuild";

await build({
    entryPoints: [
        "spec/browser/main.spec.browser.mjs",
        "spec/browser/helpers/process.mjs"
    ],
    bundle: true,
    alias: {
        util: "util",
        stream: "stream-browserify",
        path: "path-browserify",
        buffer: "buffer",
        zlib: "browserify-zlib",
        assert: "assert",
        events: "events"
    },
    outdir: "spec/browser/dist"
});
