import { assert, assertEquals } from "@std/assert";
import init, { Replay } from "./pkg/sc2_parser.js";

await init();

Deno.test(function keysExist() {
  const expectedFilePath = "test";
  const blob = Deno.readFileSync(
    "/mnt/c/Users/zacha/Documents/StarCraft II/Accounts/50968896/1-S2-1-2508124/Replays/Multiplayer/Wilshire vs EleizzeR - Hecate LE - WTF insane very close BC defense.SC2Replay",
  );
  const replay = new Replay(blob, expectedFilePath);
  assertEquals(replay.file_path, expectedFilePath, "Parsed replay's file name was incorrect.")
  assertEquals(replay.content_hash, "2c1d90cd33270e843a4be11f7fc685558b8c42553aaa701543fdc22332cb2fcb", "Parsed replay's checksum was incorrect.");
});

Deno.test(function instanceOf() {
  const blob = Deno.readFileSync(
    "/mnt/c/Users/zacha/Documents/StarCraft II/Accounts/50968896/1-S2-1-2508124/Replays/Multiplayer/Wilshire vs EleizzeR - Hecate LE - WTF insane very close BC defense.SC2Replay",
  );
  const replay = new Replay(blob, "test replay");
  assert(replay instanceof Replay, "Parsed replay was not an instance of Replay class.");
});
