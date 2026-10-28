Sekiro co-op (sekiro-coop, experimental build)
==============================================

What it is: a test build of an unofficial co-op mod for Sekiro: Shadows Die Twice
(Steam, version 1.06). Your friend appears in your world as a Wolf, moved by the game's own remote-player code.
Connection goes through Steam, so no IP addresses or port forwarding are needed.
It is NOT finished: enemies and story are not shared yet. Expect bugs and crashes.

You need
- Sekiro on Steam, version 1.06, Steam running and logged in.
- me3 (mod loader), official releases: https://github.com/garyttierney/me3/releases
- This folder anywhere on your disk.

Step 1. Find your SteamID64
  Run play-steam.cmd with no arguments, load into the game, quit.
  The window prints a line with "own id" and a 17-digit number: that is your SteamID64.
  (It is also shown on your Steam profile page URL if you use the number form.)
  Send it to your friend and get theirs.

Step 2. Meet in the same place
  Both rest at the same Sculptor's Idol before starting (the other player is placed by their
  real position, so you must be in the same area). The Wolf you see wears YOUR equipment for now.

Step 3. Play
  The host (whose world it is):  play-steam.cmd host 7656119xxxxxxxxxx
  The other player:              play-steam.cmd client 7656119xxxxxxxxxx
  (each puts the OTHER player's SteamID64)

Saves
  The mod plays on a separate save file, coop.sl2, in your normal Sekiro save folder.
  The first time it is a new game. To use your progress, copy your own S0000.sl2 to coop.sl2
  in %APPDATA%\Sekiro\<your id>\ yourself (keep a backup).

If something goes wrong
  Send the file sekiro-coop.log from this folder.

Not included: no game files. Source: sekiro-coop (AGPL-3.0-or-later), based on mstampfli/sekiro-coop; this build: https://github.com/vano112211-man/sekiro-coop/tree/coop/puppet-probe Under the AGPL, whoever shares this build must also offer its source code.
