import { assert, assertArrayIncludes } from "@std/assert";
import init, { Replay } from "./pkg/sc2_parser.js";

await init();

Deno.test(function keysExist() {
  const blob = Deno.readFileSync(
    "/mnt/c/Users/zacha/Documents/StarCraft II/Accounts/50968896/1-S2-1-2508124/Replays/Multiplayer/Wilshire vs EleizzeR - Hecate LE - WTF insane very close BC defense.SC2Replay",
  );
  const replay = new Replay(blob, "test replay");
  assertArrayIncludes(Object.keys(replay), [
    "file_path",
    "content_hash",
    "parsed",
  ]);
});

Deno.test.ignore(function instanceOf() {
  const blob = Deno.readFileSync(
    "/mnt/c/Users/zacha/Documents/StarCraft II/Accounts/50968896/1-S2-1-2508124/Replays/Multiplayer/Wilshire vs EleizzeR - Hecate LE - WTF insane very close BC defense.SC2Replay",
  );
  const replay = new Replay(blob, "test replay");
  assert(replay instanceof Replay);
});
