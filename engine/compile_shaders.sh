#!/bin/bash
set -e

# Create shaders directory if it doesn't exist
mkdir -p shaders

echo "Compiling vertex shader..."
glslc shaders/triangle.vert -o shaders/triangle.vert.spv
echo "Vertex shader compiled successfully"

echo "Compiling fragment shader..."
glslc shaders/triangle.frag -o shaders/triangle.frag.spv
echo "Fragment shader compiled successfully"

echo "All shaders compiled successfully"