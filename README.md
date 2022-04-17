# Railway

Railway is a binary file format for vector animated pictures.
This is a library for parsing, computing and rendering such pictures.

![generated.png](https://github.com/NathanRoyer/railway/blob/main/generated.png?raw=true)

# Anatomy of a railway file

Throughout this format we use the term "Couple" which means a pair of floating point numbers.
For a more exhaustive description of the file format, [see the format specification](https://github.com/NathanRoyer/railway/blob/main/format.txt).

## Virtual Machine Program

Before drawing, railway files have a program that needs to be computed to yield results.
These results are then used as coordinates for the drawing stage.
A stack is used to store these results.

### Parameters (Arguments)

Couples whose values can be changed.
Some have names, which implies that they're meant to be changed.
Those without a name are to be seen as constant values of the program.
The parameters corresponds to the initial content of the stack.

### Instructions

Mathematical operations which have one output and up to 3 inputs.
Inputs are specified using a stack index/offset.
Every instruction results in a couple which is pushed onto the stack.

### Outputs

Some results can be useful to the client code, for instance to overlay something precisely over the rendered picture.
These are specified in the file under this section, as a name and a stack index/offset.

## Draw Operations

Once every instruction has been computed, the drawing stage can begin.
It will make use of couples present in the stack, referenced by a stack index/offset.

A railway file contains a number of "layers": combinations of background triangles and masks.

### Triangles

Background triangles specify x/y coordinates and an RGBA color for each point.

### Masks

Masks are polygons constructed with arcs and bezier curves (cubic / quadratic / linear).
Bezier curves are specified using 2, 3 or 4 points.
Arcs are specified using a center, two absolute angles, and two radii.

### Clips

A clip specified a mask and a background triangle.
Every pixel that is in both the mask and the triangle will be drawn.

### Strokes

A stroke is the drawing of a mask's contour in a specified color and with a specified pattern.
