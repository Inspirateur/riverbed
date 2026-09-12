// TODO: implement a structure generation thread that listens to ChunkEvent::Added
// and load structures when they have enough loaded chunks around them (depends on the structure size)
// Structures need to be stored either in their chunks or in World directly
// (they are currently discarded at column generation stage)
// Package it in a bevy "StructurePlugin"
