// Thin wrapper over the `stellar` CLI for the TESTNET demo scripts. Keys stay
// in the CLI's keystore: scripts pass identity names, never secrets.

import { execFile } from "node:child_process";
import { promisify } from "node:util";

const run = promisify(execFile);

export const NETWORK = "testnet";
export const RPC_URL = "https://soroban-testnet.stellar.org";
export const TESTNET_PASSPHRASE = "Test SDF Network ; September 2015";
export const expertTx = (hash: string) => `https://stellar.expert/explorer/testnet/tx/${hash}`;
export const expertContract = (id: string) =>
  `https://stellar.expert/explorer/testnet/contract/${id}`;

/** Transaction hashes the CLI reports on stderr ("Signing transaction: <hash>"). */
export function txHashes(stderr: string): string[] {
  return [...stderr.matchAll(/Signing transaction: ([0-9a-f]{64})/g)].map((m) => m[1]!);
}

/** Refuses to continue unless RPC reports the testnet passphrase. */
export async function assertTestnet(): Promise<void> {
  const res = await fetch(RPC_URL, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getNetwork" }),
  });
  const body = (await res.json()) as { result?: { passphrase?: string } };
  if (body.result?.passphrase !== TESTNET_PASSPHRASE) {
    throw new Error(`refusing to run: ${RPC_URL} is not testnet`);
  }
}

export async function latestLedger(): Promise<number> {
  const res = await fetch(RPC_URL, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getLatestLedger" }),
  });
  const body = (await res.json()) as { result: { sequence: number } };
  return body.result.sequence;
}

export interface CliResult {
  out: string;
  hashes: string[];
}

/**
 * Runs `stellar <args>` with `--network testnet` added before any `--`.
 * `local` commands (e.g. `contract build`) take no network.
 */
export async function stellar(
  args: string[],
  opts: { cwd?: string; local?: boolean } = {},
): Promise<CliResult> {
  const split = args.indexOf("--");
  const full = opts.local
    ? args
    : split === -1
      ? [...args, "--network", NETWORK]
      : [...args.slice(0, split), "--network", NETWORK, ...args.slice(split)];
  try {
    const { stdout, stderr } = await run("stellar", full, { cwd: opts.cwd, maxBuffer: 16 << 20 });
    return { out: stdout.trim(), hashes: txHashes(stderr) };
  } catch (e) {
    const err = e as { stderr?: string; message: string };
    throw new Error(`stellar ${args.slice(0, 3).join(" ")} failed: ${err.stderr ?? err.message}`, {
      cause: e,
    });
  }
}

export async function address(identity: string): Promise<string> {
  const { stdout } = await run("stellar", ["keys", "address", identity]);
  return stdout.trim();
}
