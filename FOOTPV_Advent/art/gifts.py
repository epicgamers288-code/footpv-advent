"""From project root: blender -b -P art/gifts.py"""
import bpy,math
from pathlib import Path
from mathutils import Vector
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete()
def mat(name,metal):
 m=bpy.data.materials.new(name);m.use_nodes=True
 n=m.node_tree.nodes.get('Principled BSDF');n.inputs['Metallic'].default_value=metal;n.inputs['Roughness'].default_value=.24
 return m
paper=mat('Lacquer',.35);foil=mat('Satin ribbon',.65)
def box(name,loc,size,material,radius):
 bpy.ops.mesh.primitive_cube_add(size=1,location=loc);o=bpy.context.object;o.name=name;o.dimensions=size
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);o.data.materials.append(material)
 b=o.modifiers.new('Edge','BEVEL');b.width=radius;b.segments=4;o.modifiers.new('Normals','WEIGHTED_NORMAL')
box('Gift',(0,0,.8),(2,2,1.6),paper,.055);box('Lid',(0,0,1.64),(2.08,2.08,.27),paper,.045)
for w,d in [(2.015,.35),(.35,2.015)]:box('Wrap',(0,0,.8),(w,d,1.61),foil,.01)
for w,d in [(2.095,.36),(.36,2.095)]:box('Band',(0,0,1.645),(w,d,.28),foil,.009)
for k in range(4):
 a=.25+k*math.pi/2;verts=[]
 for j in range(57):
  t=j/56;r=.78*math.sin(math.pi*t);h=1.79+.46*math.sin(math.pi*t)+.18*math.sin(2*math.pi*t)
  for w in [-.13,.13]:verts.append((r*math.cos(a)-w*math.sin(a),r*math.sin(a)+w*math.cos(a),h))
 mesh=bpy.data.meshes.new('Silk');mesh.from_pydata(verts,[],[(2*i,2*i+1,2*i+3,2*i+2) for i in range(56)]);o=bpy.data.objects.new('Bow',mesh);bpy.context.collection.objects.link(o);mesh.materials.append(foil)
 for p in mesh.polygons:p.use_smooth=True
 o.modifiers.new('Thickness','SOLIDIFY').thickness=.018
box('Knot',(0,0,1.89),(.4,.36,.24),foil,.09)
for loc,energy,size in [((2,-4,7),750,4),((-5,-2,4),520,3),((3,4,5),820,3)]:
 bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.data.energy=energy;o.data.size=size;o.rotation_euler=(Vector((0,0,1))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(4,-6,4.7));o=bpy.context.object;o.rotation_euler=(Vector((0,0,1.12))-o.location).to_track_quat('-Z','Y').to_euler();o.data.type='ORTHO';o.data.ortho_scale=3.5
s=bpy.context.scene;s.camera=o;s.render.engine='CYCLES';s.cycles.samples=40;s.cycles.use_denoising=False;s.render.film_transparent=True;s.render.image_settings.color_mode='RGBA';s.render.resolution_x=384;s.render.resolution_y=384;s.render.resolution_percentage=100
for i,(base,rib) in enumerate([((.028,.16,.23),(.9,.53,.12)),((.035,.13,.43),(.33,.83,.98)),((.62,.30,.05),(.95,.72,.35)),((.025,.24,.18),(.57,.89,.64))]):
 for m,c in [(paper,base),(foil,rib)]:m.node_tree.nodes.get('Principled BSDF').inputs['Base Color'].default_value=(*c,1)
 s.render.image_settings.file_format='WEBP';s.render.image_settings.quality=92;s.render.filepath=str(Path.cwd()/'web/public'/f'gift-{i}.webp');bpy.ops.render.render(write_still=True)
