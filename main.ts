import init, { Replay, test } from "./pkg/sc2_parser.js";

await init();

test();

const blob = Deno.readFileSync("/mnt/c/Users/zacha/Documents/StarCraft II/Accounts/50968896/1-S2-1-2508124/Replays/Multiplayer/Wilshire vs EleizzeR - Hecate LE - WTF insane very close BC defense.SC2Replay")
const replay = new Replay(blob, "test replay", "9747440aa8bfd600cd3b4af674c0dae7613fc5c0", ["tag1", "tag2"]);

console.log(replay);