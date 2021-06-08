
const filename = Deno.args[0]

const pluginRid = Deno.openPlugin(filename);
const path = import.meta.url.substring("file:///".length);

Deno.writeTextFileSync(path + ".1.txt", "is due to start service");

await Deno.core.opAsync("op_winscm_start_dispatcher", {
    name: "foo"
});

Deno.writeTextFileSync(path + ".2.txt", "stopping service");

console.log("SUCCESS");