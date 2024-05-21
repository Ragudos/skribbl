import * as esbuild from "esbuild";
import { argv } from "process";

/**
 * @type {esbuild.BuildOptions}
 */
const config = {
    entryPoints: ["src/index.ts"],
    outdir: "../dist",
    bundle: true,
    sourcemap: true,
    format: "esm",
    splitting: true,
    legalComments: "inline",
};

if (argv.includes("--watch")) {
    (
        await esbuild.context({
            ...config,
            define: {
                "import.meta.env.DEV": "true",
                "import.meta.env.PROD": "false",
            },
        })
    ).watch();
} else {
    await esbuild.build({
        ...config,
        minify: true,
        define: {
            "import.meta.env.DEV": "false",
            "import.meta.env.PROD": "true",
        },
    });
}
