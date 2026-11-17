import bpy, json, sys, math, traceback
from pathlib import Path
from mathutils import Vector

def mat(name, color, metallic=0.0, roughness=0.45, subsurface=0.0):
    m=bpy.data.materials.get(name)
    if m is not None: return m
    m=bpy.data.materials.new(name); m.diffuse_color=(*color,1); m.use_nodes=True
    p=m.node_tree.nodes.get("Principled BSDF")
    p.inputs["Base Color"].default_value=(*color,1)
    p.inputs["Metallic"].default_value=metallic
    p.inputs["Roughness"].default_value=roughness
    if "Subsurface Weight" in p.inputs: p.inputs["Subsurface Weight"].default_value=subsurface
    return m

def uv(name, loc, scale, material, seg=48, rings=32, smooth=True):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=seg, ring_count=rings, location=loc)
    o=bpy.context.object; o.name=name; o.scale=scale
    if material: o.data.materials.append(material)
    if smooth:
        for p in o.data.polygons: p.use_smooth=True
    return o

def cone(name, loc, radius1, radius2, depth, material, vertices=48, rotation=(0,0,0)):
    bpy.ops.mesh.primitive_cone_add(vertices=vertices, radius1=radius1, radius2=radius2, depth=depth, location=loc, rotation=rotation)
    o=bpy.context.object; o.name=name
    if material:o.data.materials.append(material)
    for p in o.data.polygons:p.use_smooth=True
    bevel=o.modifiers.new("Soft crafted edges","BEVEL"); bevel.width=0.025; bevel.segments=3
    o.modifiers.new("Weighted corner normals","WEIGHTED_NORMAL")
    return o

def curve(name, points, bevel, material, resolution=16):
    c=bpy.data.curves.new(name,"CURVE"); c.dimensions="3D"; c.resolution_u=resolution; c.bevel_depth=bevel; c.bevel_resolution=5
    s=c.splines.new("BEZIER"); s.bezier_points.add(len(points)-1)
    for b,co in zip(s.bezier_points,points):
        b.co=co; b.handle_left_type="AUTO"; b.handle_right_type="AUTO"
    o=bpy.data.objects.new(name,c); bpy.context.collection.objects.link(o)
    if material:o.data.materials.append(material)
    return o

def setup_scene(outdir, prefix):
    sc=bpy.context.scene
    sc.render.engine="CYCLES"
    sc.cycles.samples=32
    sc.cycles.use_denoising=True
    sc.render.resolution_x=1200; sc.render.resolution_y=1460; sc.render.resolution_percentage=100
    sc.view_settings.view_transform="AgX"
    world=bpy.data.worlds.new("RevEngine Studio World") if not bpy.data.worlds else bpy.data.worlds[0]
    sc.world=world; world.use_nodes=True
    world.node_tree.nodes["Background"].inputs[0].default_value=(0.055,0.075,0.12,1)
    world.node_tree.nodes["Background"].inputs[1].default_value=0.35
    bpy.ops.object.camera_add(location=(7,-11,5.5))
    cam=bpy.context.object; cam.name="Portrait Camera"; cam.data.lens=62
    target=Vector((0,0,2.2)); cam.rotation_euler=(target-cam.location).to_track_quat("-Z","Y").to_euler(); sc.camera=cam
    def area(name,loc,power,color,size):
        bpy.ops.object.light_add(type="AREA", location=loc)
        l=bpy.context.object; l.name=name; l.data.energy=power; l.data.shape="DISK"; l.data.size=size; l.data.color=color
        l.rotation_euler=(target-l.location).to_track_quat("-Z","Y").to_euler()
    area("Key softbox", (4,-6,8), 1400, (1.0,0.78,0.58), 5)
    area("Cool fill",(-5,-3,4),1000,(0.42,0.68,1.0),4)
    area("Golden rim",(1,4,6),1900,(1.0,0.48,0.18),3)
    sc.render.image_settings.file_format="PNG"
    sc.render.filepath=str(outdir/(prefix+"_render.png"))
    sc.camera.data.dof.use_dof=True; sc.camera.data.dof.focus_distance=11; sc.camera.data.dof.aperture_fstop=5.6

def create_elf(outdir):
    bpy.ops.object.select_all(action="SELECT"); bpy.ops.object.delete(use_global=False)
    skin=mat("Warm ivory skin",(0.62,0.37,0.23),0,0.34,0.13)
    skinlight=mat("Ear highlights",(0.78,0.49,0.31),0,0.4,0.1)
    emerald=mat("Enchanted emerald velvet",(0.018,0.16,0.095),0.12,0.31)
    darkgreen=mat("Deep forest cloth",(0.008,0.055,0.035),0.05,0.52)
    gold=mat("Antique elven gold",(0.72,0.42,0.10),0.82,0.22)
    leather=mat("Supple dark leather",(0.105,0.035,0.018),0.05,0.36)
    hair=mat("Copper auburn hair",(0.19,0.045,0.018),0.08,0.29)
    hairlight=mat("Hair highlights",(0.48,0.16,0.045),0.22,0.3)
    white=mat("Eye ivory",(0.83,0.82,0.69),0,0.2)
    iris=mat("Jade iris",(0.035,0.32,0.18),0.25,0.2)
    black=mat("Pupil",(0.006,0.009,0.012),0,0.12)
    cloth=uv("Tailored tunic",(0,0,2.05),(0.72,0.42,0.9),emerald)
    uv("Neck",(0,0,2.96),(0.22,0.22,0.34),skin)
    uv("Face",(0,-0.02,3.42),(0.53,0.43,0.68),skin,64,48)
    # tapered jaw/chin
    uv("Chin",(0,-0.18,3.08),(0.34,0.31,0.26),skin)
    # pointed ears, inner ear and tips
    for side in [-1,1]:
        ear=uv("Pointed ear",(side*0.55,0,3.5),(0.38,0.13,0.19),skin)
        ear.rotation_euler[1]=side*-0.28
        tip=cone("Elven ear tip",(side*0.84,0,3.58),0.16,0,0.65,skinlight,32,(0,side*-math.pi/2.8,0))
        tip.scale=(1,0.65,0.55)
        uv("Inner ear",(side*0.61,-0.115,3.53),(0.20,0.035,0.075),skinlight)
    # eyes, brows, nose, mouth
    for side in [-1,1]:
        x=side*0.205
        uv("Eye socket",(x,-0.397,3.49),(0.155,0.065,0.105),leather)
        uv("Eye",(x,-0.445,3.49),(0.125,0.052,0.073),white)
        uv("Iris",(x+side*0.012,-0.49,3.49),(0.052,0.022,0.057),iris)
        uv("Pupil",(x+side*0.012,-0.507,3.49),(0.024,0.012,0.034),black)
        uv("Eye glint",(x+side*0.025,-0.519,3.515),(0.012,0.008,0.012),white)
        curve("Expressive brow",[(x-side*0.12,-0.43,3.64),(x,-0.46,3.68),(x+side*0.12,-0.42,3.64)],0.027,hair)
    uv("Nose bridge",(0,-0.45,3.36),(0.075,0.12,0.19),skinlight)
    uv("Nose tip",(0,-0.55,3.29),(0.10,0.075,0.065),skin)
    curve("Gentle smile",[(-0.12,-0.444,3.17),(0,-0.47,3.145),(0.12,-0.444,3.17)],0.014,leather)
    # flowing hair cap and locks
    uv("Hair crown",(0,0.01,3.91),(0.55,0.47,0.31),hair)
    for side in [-1,1]:
        for i in range(5):
            x=side*(0.28+i*0.045)
            curve("Long flowing hair lock",[(x,0.0,3.92),(x+side*0.10,0.10,3.55),(x+side*0.12,0.15,3.10-i*0.025)],0.055-(i*0.004),hair if i%2 else hairlight)
    for i in range(7):
        x=-0.28+i*0.09
        curve("Forehead fringe",[(x, -0.30,3.95),(x+0.05,-0.42,3.78),(x+0.02,-0.42,3.65)],0.045,hair if i%2 else hairlight)
    # shoulders, sleeves, arms, hands
    for side in [-1,1]:
        uv("Shoulder pauldron",(side*0.68,0,2.63),(0.31,0.39,0.28),darkgreen)
        arm=uv("Sleeve",(side*0.86,0,2.18),(0.24,0.29,0.62),emerald); arm.rotation_euler[1]=side*-0.24
        uv("Hand",(side*1.02,-0.03,1.55),(0.14,0.14,0.20),skin)
        for j in range(3):
            curve("Finger",[(side*(0.98+j*0.035),-0.13,1.5),(side*(1.0+j*0.035),-0.15,1.39)],0.025,skin)
        uv("Golden pauldron trim",(side*0.68,-0.29,2.65),(0.29,0.08,0.05),gold)
    # collar, chest embroidery, belt
    cone("High collar",(0,-0.02,2.87),0.35,0.20,0.34,gold,64)
    uv("Belt",(0,-0.01,1.55),(0.72,0.45,0.12),leather)
    uv("Belt buckle",(0,-0.47,1.57),(0.14,0.035,0.14),gold)
    for side in [-1,1]:
        curve("Tunic embroidery",[(side*0.12,-0.42,2.75),(side*0.28,-0.44,2.45),(side*0.18,-0.44,2.10)],0.018,gold)
    # legs, boots
    for side in [-1,1]:
        uv("Trousers",(side*0.32,0,0.92),(0.28,0.33,0.68),darkgreen)
        uv("Leather boot",(side*0.34,-0.12,0.26),(0.31,0.48,0.27),leather)
        uv("Boot gold clasp",(side*0.34,-0.52,0.38),(0.12,0.025,0.05),gold)
    # pointed forest hat with gold band
    cone("Tall elven cap",(0,0.05,4.34),0.55,0.025,1.15,emerald,96,(0.08,0.08,0))
    uv("Hat brim",(0,0.03,3.98),(0.62,0.5,0.10),darkgreen)
    uv("Hat gold band",(0,0.01,4.05),(0.52,0.43,0.045),gold)
    uv("Hat jewel",(0,-0.45,4.16),(0.095,0.05,0.13),iris)
    # cloak behind
    cloak=uv("Draped cloak",(0,0.28,2.1),(0.83,0.24,1.18),darkgreen)
    # engraved jewelry
    for z in [2.72,2.60,2.48]:
        uv("Tiny gold filigree",(0,-0.47,z),(0.035,0.018,0.035),gold)
    setup_scene(outdir,"revengine_elf")
    sc=bpy.context.scene
    sc.render.resolution_x=1200; sc.render.resolution_y=1460
    sc.render.filepath=str(outdir/"revengine_elf_portrait.png")
    bpy.ops.wm.save_as_mainfile(filepath=str(outdir/"revengine_elf.blend"))
    bpy.ops.render.render(write_still=True)
    bpy.ops.export_scene.gltf(filepath=str(outdir/"revengine_elf.glb"),export_format="GLB",export_materials="EXPORT")
    return ["revengine_elf.blend","revengine_elf_portrait.png","revengine_elf.glb"]

def run_plan(plan_path, project):
    # PowerShell 5 writes UTF-8 with BOM; accept both BOM and BOM-less JSON plans.
    plan=json.loads(Path(plan_path).read_text(encoding="utf-8-sig"))
    out=Path(project)/"Assets"/"Generated"; out.mkdir(parents=True,exist_ok=True)
    done=[]
    actions=plan.get("actions",[])
    if not isinstance(actions,list) or not actions:
        raise ValueError("O plano Blender não contém operações válidas.")
    for index,a in enumerate(actions):
        if not isinstance(a,dict):
            raise TypeError("Operação Blender %d não é um objeto JSON." % index)
        # RevAI historically emitted `action`; examples and older plans use `type`.
        # Accept both so the model contract and the executor cannot silently diverge.
        kind=str(a.get("type") or a.get("action") or "").strip().lower()
        if not kind:
            raise ValueError("Operação Blender %d sem campo 'type'/'action'." % index)
        if kind=="create_elf":
            done.extend(create_elf(out))
        elif kind=="new_scene":
            bpy.ops.object.select_all(action="SELECT"); bpy.ops.object.delete(use_global=False); done.append("scene_cleared")
        elif kind=="add_mesh":
            shape=a.get("shape","cube"); loc=a.get("location",[0,0,0]); size=float(a.get("size",2))
            detail=max(1,min(6,int(a.get("detail_level",3))))
            ops={"cube":lambda:bpy.ops.mesh.primitive_cube_add(size=size,location=loc),"sphere":lambda:bpy.ops.mesh.primitive_uv_sphere_add(segments=64*detail,ring_count=32*detail,radius=size/2,location=loc),"cylinder":lambda:bpy.ops.mesh.primitive_cylinder_add(vertices=64*detail,radius=size/2,depth=size,location=loc),"cone":lambda:bpy.ops.mesh.primitive_cone_add(vertices=64*detail,radius1=size/2,radius2=0,depth=size,location=loc),"torus":lambda:bpy.ops.mesh.primitive_torus_add(major_segments=96*detail,minor_segments=24*detail,location=loc),"monkey":lambda:bpy.ops.mesh.primitive_monkey_add(location=loc)}
            ops.get(shape,ops["cube"])(); o=bpy.context.object; o.name=str(a.get("name","Generated_"+shape))
            if a.get("smooth",True):
                for poly in o.data.polygons: poly.use_smooth=True
            if shape in ("cube","monkey"):
                sub=o.modifiers.new("Micro-surface subdivision","SUBSURF"); sub.levels=min(3,detail); sub.render_levels=min(4,detail)
            bevel=o.modifiers.new("Precision edge bevel","BEVEL"); bevel.width=size*0.008; bevel.segments=3
            o.modifiers.new("Weighted vertex normals","WEIGHTED_NORMAL")
            if a.get("apply_uv",True) and o.type=="MESH":
                bpy.context.view_layer.objects.active=o; o.select_set(True)
                bpy.ops.object.mode_set(mode="EDIT"); bpy.ops.mesh.select_all(action="SELECT"); bpy.ops.uv.smart_project(island_margin=0.015); bpy.ops.object.mode_set(mode="OBJECT")
            done.append(o.name)
        elif kind=="generate_terrain":
            loc=a.get("location",[0,0,0]); size=float(a.get("size",80)); resolution=max(32,min(512,int(a.get("resolution",192))))
            import mathutils
            verts=[]; faces=[]
            for y in range(resolution+1):
                for x in range(resolution+1):
                    u=x/resolution-0.5; v=y/resolution-0.5
                    n=(math.sin(u*31+math.cos(v*17))*0.42+math.cos(v*43+u*9)*0.24+math.sin((u+v)*83)*0.08)
                    verts.append((loc[0]+u*size,loc[1]+v*size,loc[2]+n*float(a.get("height",5))))
            for y in range(resolution):
                for x in range(resolution):
                    i=y*(resolution+1)+x; faces.append((i,i+1,i+resolution+2,i+resolution+1))
            mesh=bpy.data.meshes.new("High density terrain mesh"); mesh.from_pydata(verts,[],faces); mesh.update()
            obj=bpy.data.objects.new(str(a.get("name","Procedural_Terrain")),mesh); bpy.context.collection.objects.link(obj)
            for poly in mesh.polygons: poly.use_smooth=True
            obj["revengine_vertices"]=len(verts); obj["revengine_faces"]=len(faces)
            sub=obj.modifiers.new("Terrain smoothing","SUBSURF"); sub.levels=1
            done.append(obj.name+" ("+str(len(verts))+" vertices)")
        elif kind=="import_asset":
            asset=Path(a.get("path",""))
            if not asset.is_file(): raise FileNotFoundError(str(asset))
            ext=asset.suffix.lower()
            if ext==".glb" or ext==".gltf": bpy.ops.import_scene.gltf(filepath=str(asset))
            elif ext==".fbx": bpy.ops.import_scene.fbx(filepath=str(asset),use_image_search=True)
            elif ext==".obj": bpy.ops.wm.obj_import(filepath=str(asset))
            else: raise ValueError("Formato de asset não suportado: "+ext)
            imported=list(bpy.context.selected_objects)
            for obj in imported:
                if obj.type=="MESH":
                    obj["revengine_imported"]=True
                    obj["revengine_vertices"]=len(obj.data.vertices)
                    obj["revengine_faces"]=len(obj.data.polygons)
                    # Normalize the imported asset for a predictable editable Blender workflow.
                    if len(obj.data.vertices) and not obj.data.uv_layers:
                        bpy.ops.object.select_all(action="DESELECT")
                        obj.select_set(True); bpy.context.view_layer.objects.active=obj
                        bpy.ops.object.mode_set(mode="EDIT")
                        bpy.ops.mesh.select_all(action="SELECT")
                        bpy.ops.uv.smart_project(island_margin=0.012)
                        bpy.ops.object.mode_set(mode="OBJECT")
                    if not obj.data.materials:
                        fallback=mat("RevEngine Auto PBR - "+obj.name,(0.72,0.74,0.78),0.0,0.58)
                        obj.data.materials.append(fallback)
                    for slot in obj.material_slots:
                        if slot.material and slot.material.use_nodes:
                            bsdf=slot.material.node_tree.nodes.get("Principled BSDF")
                            if bsdf:
                                if "Metallic" in bsdf.inputs: bsdf.inputs["Metallic"].default_value=max(0.0,min(1.0,bsdf.inputs["Metallic"].default_value))
                                if "Roughness" in bsdf.inputs: bsdf.inputs["Roughness"].default_value=max(0.04,min(1.0,bsdf.inputs["Roughness"].default_value))
            done.extend([o.name for o in imported])
        elif kind=="add_material":
            o=bpy.data.objects.get(a.get("object",""))
            if o:
                m=mat(a.get("name","AI Material"),a.get("color",[0.2,0.4,0.8]),float(a.get("metallic",0)),float(a.get("roughness",0.4)))
                o.data.materials.append(m); done.append("material:"+m.name)
        elif kind=="add_pbr_material":
            o=bpy.data.objects.get(a.get("object",""))
            if o and o.type=="MESH":
                m=mat(a.get("name","Auto PBR Material"),a.get("color",[0.8,0.8,0.8]),float(a.get("metallic",0)),float(a.get("roughness",0.6)))
                nodes=m.node_tree.nodes; links=m.node_tree.links; bsdf=nodes.get("Principled BSDF")
                for channel,path in a.get("textures",{}).items():
                    image_path=Path(path)
                    if not image_path.is_file(): continue
                    image=bpy.data.images.load(str(image_path),check_existing=True)
                    tex=nodes.new("ShaderNodeTexImage"); tex.image=image; tex.label=channel
                    if channel.lower() in ("albedo","basecolor","diffuse","color"):
                        links.new(tex.outputs["Color"],bsdf.inputs["Base Color"])
                    elif channel.lower() in ("roughness","rough"):
                        tex.image.colorspace_settings.name="Non-Color"; links.new(tex.outputs["Color"],bsdf.inputs["Roughness"])
                    elif channel.lower() in ("metallic","metalness"):
                        tex.image.colorspace_settings.name="Non-Color"; links.new(tex.outputs["Color"],bsdf.inputs["Metallic"])
                    elif channel.lower() in ("normal","normal_gl","nor_gl"):
                        tex.image.colorspace_settings.name="Non-Color"; normal=nodes.new("ShaderNodeNormalMap"); links.new(tex.outputs["Color"],normal.inputs["Color"]); links.new(normal.outputs["Normal"],bsdf.inputs["Normal"])
                if len(o.data.materials): o.data.materials[0]=m
                else: o.data.materials.append(m)
                done.append("pbr_material:"+m.name)
        elif kind=="create_animation_preset":
            # Procedural animation presets work on a selected/imported armature or a fallback object.
            target=bpy.data.objects.get(a.get("object",""))
            if target is None: target=bpy.context.active_object
            preset=str(a.get("preset","idle")).lower()
            fps=max(12,min(120,int(a.get("fps",30))))
            duration=max(1.0,min(12.0,float(a.get("duration",2.0))))
            frames=max(2,int(fps*duration))
            action_name=str(a.get("name", "RevEngine_"+preset.title()))
            if target is None: raise ValueError("Selecione um personagem/objeto para gerar a animação.")
            if target.type=="ARMATURE":
                target.animation_data_create(); action=bpy.data.actions.new(action_name); target.animation_data.action=action
                bones=list(target.pose.bones)
                def bone_match(*terms):
                    return next((b for b in bones if any(t in b.name.lower() for t in terms)),None)
                spine=bone_match("spine","chest","torso"); head=bone_match("head","neck")
                upper_arms=[b for b in bones if ("arm" in b.name.lower() or "shoulder" in b.name.lower())]
                legs=[b for b in bones if any(t in b.name.lower() for t in ("thigh","leg","foot"))]
                for frame in range(1,frames+1):
                    phase=(frame-1)/max(1,frames-1)*math.tau
                    bpy.context.scene.frame_set(frame)
                    if preset in ("walk","run"):
                        speed=2.0 if preset=="run" else 1.0
                        for i,b in enumerate(legs):
                            side=-1 if i%2==0 else 1
                            b.rotation_euler.x=math.sin(phase*2)*0.48*speed*side
                            b.keyframe_insert(data_path="rotation_euler",frame=frame)
                        for i,b in enumerate(upper_arms):
                            side=-1 if i%2==0 else 1
                            b.rotation_euler.x=-math.sin(phase*2)*0.38*speed*side
                            b.keyframe_insert(data_path="rotation_euler",frame=frame)
                        if spine:
                            spine.location.z=abs(math.sin(phase*2))*0.045*speed
                            spine.keyframe_insert(data_path="location",frame=frame)
                    elif preset in ("jump","attack"):
                        pulse=max(0.0,math.sin(phase))
                        for b in legs+upper_arms:
                            b.rotation_euler.x=(0.7 if preset=="jump" else -0.9)*pulse
                            b.keyframe_insert(data_path="rotation_euler",frame=frame)
                    else:
                        if spine:
                            spine.location.z=math.sin(phase)*0.025
                            spine.keyframe_insert(data_path="location",frame=frame)
                        if head:
                            head.rotation_euler.z=math.sin(phase*0.5)*0.035
                            head.keyframe_insert(data_path="rotation_euler",frame=frame)
                # Blender 5.x uses layered Actions; keyframe insertion already defaults to Bezier.
            else:
                target.animation_data_create(); action=bpy.data.actions.new(action_name); target.animation_data.action=action
                for frame in range(1,frames+1):
                    phase=(frame-1)/max(1,frames-1)*math.tau
                    bpy.context.scene.frame_set(frame)
                    if preset in ("walk","run"):
                        target.location.z=abs(math.sin(phase*2))*(0.08 if preset=="run" else 0.035)
                        target.rotation_euler.y=math.sin(phase)*0.08
                    elif preset=="jump":
                        target.location.z=max(0.0,math.sin(phase))*1.8
                    elif preset=="attack":
                        target.rotation_euler.x=math.sin(phase)*0.8
                    else:
                        target.location.z=math.sin(phase)*0.035
                    target.keyframe_insert(data_path="location",frame=frame)
                    target.keyframe_insert(data_path="rotation_euler",frame=frame)
            bpy.context.scene.frame_start=1; bpy.context.scene.frame_end=frames
            done.append("animation_preset:"+action_name+":"+preset)
        elif kind=="generate_environment":
            # High-density procedural map generator: multi-octave terrain, biome scatter, water and paths.
            size=max(20.0,min(500.0,float(a.get("size",120))))
            resolution=max(64,min(512,int(a.get("resolution",256))))
            height=max(1.0,min(80.0,float(a.get("height",18))))
            biome=str(a.get("biome","forest")).lower()
            verts=[]; faces=[]
            for y in range(resolution+1):
                for x in range(resolution+1):
                    u=x/resolution-0.5; v=y/resolution-0.5
                    n=(math.sin(u*17+math.cos(v*13))*0.34+math.cos(v*31+u*11)*0.22+
                       math.sin((u+v)*59)*0.12+math.cos((u-v)*103)*0.055+
                       math.sin(u*211+v*173)*0.018)
                    ridge=abs(math.sin(u*8+v*5))*0.18
                    z=(n+ridge)*height
                    if biome in ("desert","canyon"): z*=0.55
                    verts.append((u*size,v*size,z))
            for y in range(resolution):
                for x in range(resolution):
                    i=y*(resolution+1)+x
                    faces.append((i,i+1,i+resolution+2,i+resolution+1))
            mesh=bpy.data.meshes.new("RevEngine_HighDensity_Environment_Mesh")
            mesh.from_pydata(verts,[],faces); mesh.update()
            terrain=bpy.data.objects.new("RevEngine_Procedural_"+biome.title()+"_Terrain",mesh)
            bpy.context.collection.objects.link(terrain)
            terrain["revengine_vertices"]=len(verts); terrain["revengine_faces"]=len(faces)
            terrain["revengine_biome"]=biome
            for p in mesh.polygons: p.use_smooth=True
            terrain.data.materials.append(mat("Terrain - "+biome,(0.12,0.24,0.12) if biome=="forest" else (0.48,0.34,0.19) if biome in ("desert","canyon") else (0.28,0.34,0.42),0.0,0.88))
            terrain.modifiers.new("Terrain micro-normal smoothing","WEIGHTED_NORMAL")
            # deterministic scatter of rocks/trees; preserve editable instances as mesh objects.
            import random
            rng=random.Random(int(a.get("seed",42)))
            count=max(0,min(600,int(a.get("scatter",180))))
            for i in range(count):
                x=rng.uniform(-size*0.46,size*0.46); y=rng.uniform(-size*0.46,size*0.46)
                if abs(x)<size*0.08 and abs(y)<size*0.08: continue
                z=math.sin(x/size*17+math.cos(y/size*13))*height*0.25
                if biome=="forest" and i%3:
                    bpy.ops.mesh.primitive_cone_add(vertices=12,radius1=rng.uniform(0.7,1.8),radius2=0,depth=rng.uniform(3,8),location=(x,y,z+2))
                    obj=bpy.context.object; obj.name="Pine_Tree_%03d"%i
                    obj.data.materials.append(mat("Pine foliage",(0.035,0.19,0.075),0,0.8))
                    bpy.ops.mesh.primitive_cylinder_add(vertices=10,radius=0.22,depth=1.8,location=(x,y,z+0.9))
                    bpy.context.object.name="Pine_Trunk_%03d"%i
                else:
                    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2,radius=rng.uniform(0.5,2.0),location=(x,y,z+0.4))
                    bpy.context.object.name="Environment_Rock_%03d"%i
            done.append("environment:"+terrain.name+":"+str(len(verts))+" vertices")
        elif kind=="create_animation":
            o=bpy.data.objects.get(a.get("object",""))
            if o:
                action_name=str(a.get("name","RevEngine_Action"))
                o.animation_data_create(); o.animation_data.action=bpy.data.actions.new(action_name)
                for key in a.get("keyframes",[]):
                    frame=int(key.get("frame",1)); bpy.context.scene.frame_set(frame)
                    if "location" in key: o.location=key["location"]; o.keyframe_insert(data_path="location",frame=frame)
                    if "rotation" in key: o.rotation_euler=key["rotation"]; o.keyframe_insert(data_path="rotation_euler",frame=frame)
                    if "scale" in key: o.scale=key["scale"]; o.keyframe_insert(data_path="scale",frame=frame)
                done.append("animation:"+action_name)
        elif kind=="add_camera":
            bpy.ops.object.camera_add(location=a.get("location",[7,-7,5])); bpy.context.scene.camera=bpy.context.object; done.append("camera")
        elif kind=="add_light":
            bpy.ops.object.light_add(type=a.get("light_type","AREA"),location=a.get("location",[4,-4,6]))
            l=bpy.context.object; l.data.energy=float(a.get("power",1000)); done.append("light:"+l.name)
        elif kind=="render":
            setup_scene(out,"revengine_asset"); bpy.ops.render.render(write_still=True); done.append("render:"+str(bpy.context.scene.render.filepath))
        elif kind=="export":
            fmt=a.get("format","GLB").upper()
            path=out/("generated_asset."+("glb" if fmt=="GLB" else "fbx" if fmt=="FBX" else "obj"))
            if fmt=="GLB": bpy.ops.export_scene.gltf(filepath=str(path),export_format="GLB")
            elif fmt=="FBX": bpy.ops.export_scene.fbx(filepath=str(path))
            elif fmt=="OBJ": bpy.ops.wm.obj_export(filepath=str(path))
            done.append(str(path))
        else:
            raise ValueError("Operação Blender não reconhecida: %r (índice %d)." % (kind,index))
        print("REVENGINE_STEP=" + str(kind) + "; total_assets=" + str(len(done)), flush=True)
    # Every AI generation receives a real inspection render, even when the model forgot
    # to request an explicit `render` action. Keep it separate from the editable viewport.
    render_path=out/"generated_scene_render.png"
    if not render_path.exists():
        setup_scene(out,"generated_scene")
        scene=bpy.context.scene
        scene.render.resolution_x=1280; scene.render.resolution_y=900
        scene.render.resolution_percentage=100
        scene.cycles.samples=24
        scene.render.filepath=str(render_path)
        bpy.ops.render.render(write_still=True)
        done.append("generated_scene_render.png")
    # Every AI generation leaves an editable native Blender source file.
    blend_path=out/"generated_scene.blend"
    bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))
    if "generated_scene.blend" not in done: done.append("generated_scene.blend")
    # Always provide a runtime-ready glTF binary unless the plan exported one already.
    glb_path=out/"generated_scene.glb"
    if not glb_path.exists():
        bpy.ops.export_scene.gltf(filepath=str(glb_path), export_format="GLB", export_materials="EXPORT")
    if "generated_scene.glb" not in done: done.append("generated_scene.glb")
    return done

def main():
    args=sys.argv[sys.argv.index("--")+1:] if "--" in sys.argv else []
    p=Path(args[args.index("--plan")+1]); project=Path(args[args.index("--project")+1])
    try:
        done=run_plan(p,project)
        print("REVENGINE_BLENDER_OK="+json.dumps(done,ensure_ascii=False))
    except Exception:
        traceback.print_exc(); sys.exit(2)
if __name__=="__main__": main()
