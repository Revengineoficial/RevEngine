# RevEngine Blender bridge - run with: blender --background --python this_file.py
import bpy, os, sys
from pathlib import Path

def main():
    args = sys.argv[sys.argv.index("--")+1:] if "--" in sys.argv else []
    out = Path(args[0]).resolve() if args else Path.cwd() / "Assets" / "blender_scene.glb"
    out.parent.mkdir(parents=True, exist_ok=True)
    # Create a clean starter scene only when explicitly requested with --revengine-create
    if "--revengine-create" in args:
        bpy.ops.object.select_all(action='SELECT')
        bpy.ops.object.delete(use_global=False)
        bpy.ops.mesh.primitive_cube_add(size=2, location=(0,0,0))
        cube=bpy.context.object
        cube.name="RevEngine_Cube"
        mat=bpy.data.materials.new("RevEngine_Studio_Material")
        mat.diffuse_color=(0.18,0.48,0.85,1)
        cube.data.materials.append(mat)
        bpy.ops.object.camera_add(location=(7,-7,5))
        bpy.context.scene.camera=bpy.context.object
        bpy.ops.object.light_add(type='AREA', location=(4,-4,6))
        bpy.context.object.data.energy=900
    bpy.ops.export_scene.gltf(filepath=str(out), export_format='GLB')
    print("REVENGINE_EXPORT_OK="+str(out))

if __name__ == "__main__":
    main()
