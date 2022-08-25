# Bevy Jam 2: Combine!

A public repository for the second Bevy Game Jam!

## Game Ideas:
- You have a card game with two decks and two hands per player. Each time you need to play a card you have to combine two halves.
- A puzzle-ish Tetris / jigsaw cross over where you combine progressively bigger pieces.
- A game where you combine ingredients for procedurally generated recipes.
- A traditional 2D game where you can combine items.
- (Chosen) A traditional 2D game where you combine two different worlds.

## MVP for idea:
- Top-down game @ 720P
- Move and interact with WASD + jkl;.
- Title screen
- New Game Screen
- Load / Save Screen
- Settings Menu

## Project Structure:
- main.rs - Where all of the magic happens.
- components - Directory with reusable general components.
- systems - The methods which work on the components.
- resources.rs - Shared resources that need to be used across sytems or components.
- level.rs - Most of the level-specific loading and entity things.  (Doors, player spawn location, etc.)
- player.rs - Player components and systems.
- slime.rs - AI, sprites, and such.  When we have more enemies this might get split out.

## LDTK Map Data:

#### Layers (Top +Z to bottom)
- OBJECTS_TOP_DECO
- OBJECTS_TOP
- [Entities]
- OBJECTS_DECO
- OBJECTS
- GROUND_DECO
- GROUND
- COLLISION

#### Entities
- PLAYER_SPAWN
- SLIME_SPAWN (FieldIdentifier "color" - Tints Slime)
- DOOR (FieldIdentifier "destination" -- The matching "target".)