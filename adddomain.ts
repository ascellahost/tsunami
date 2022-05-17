import "https://deno.land/std@0.139.0/dotenv/load.ts";
import postgres from "https://deno.land/x/postgresjs/mod.js";
const domain = prompt("Domain");
if (!domain) throw TypeError("Domain is null");
const owner = prompt("Owner");
if (!domain) throw TypeError("Owner is null");
const sql = postgres(Deno.env.get("DATABASE_URL")!);

let res = await fetch(`https://api.cloudflare.com/client/v4/zones`, {
  method: "POST",
  headers: {
    "X-Auth-Email": Deno.env.get("CLOUDFLARE_EMAIL")!,
    "x-Auth-key": Deno.env.get("CLOUDFLARE_API")!,
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    "account": { id: Deno.env.get("CLOUDFLARE_ID")! },
    name: domain,
    jump_start: true,
    "full": true,
  }),
}).then((r) => r.json());
if (!res?.result?.name_servers) {
  throw new Error(`Failed to add domain! ${JSON.stringify(res)}`);
}

await sql
  `INSERT INTO domains(domain,apex,owner) VALUES(${domain}, true, ${owner})`;
console.log("Done!");
console.log(res.result.name_servers.map((x: string) => `\`${x}\``).join("\n"));
