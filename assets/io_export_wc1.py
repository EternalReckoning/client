# ##### BEGIN GPL LICENSE BLOCK #####
#
#  This program is free software; you can redistribute it and/or
#  modify it under the terms of the GNU General Public License
#  as published by the Free Software Foundation; either version 2
#  of the License, or (at your option) any later version.
#
#  This program is distributed in the hope that it will be useful,
#  but WITHOUT ANY WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#  GNU General Public License for more details.
#
#  You should have received a copy of the GNU General Public License
#  along with this program; if not, write to the Free Software Foundation,
#  Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.
#
# ##### END GPL LICENSE BLOCK #####

# <pep8 compliant>

import os

import bpy
from mathutils import Matrix, Vector, Color
from bpy_extras import io_utils, node_shader_utils

from bpy_extras.wm_utils.progress_report import (
    ProgressReport,
    ProgressReportSubstep,
)

bl_info = {
    "name": "WorldClient v1 format",
    "author": "Henry Carlson",
    "version": (0, 0, 1),
    "blender": (2, 80, 0),
    "location": "File > Import-Export",
    "description": "Import-Export WC1, Import WC1 mesh",
    "warning": "",
    "wiki_url": "",
    "category": "Import-Export"}

if "bpy" in locals():
    import importlib
    if "export_wc1" in locals():
        importlib.reload(export_obj)


import bpy
from bpy.props import (
        BoolProperty,
        FloatProperty,
        StringProperty,
        EnumProperty,
        )
from bpy_extras.io_utils import (
        ImportHelper,
        ExportHelper,
        orientation_helper,
        path_reference_mode,
        axis_conversion,
        )


@orientation_helper(axis_forward='-Z', axis_up='Y')
class ExportWC1(bpy.types.Operator, ExportHelper):
    """Save a WC1 File"""

    bl_idname = "export_scene.wc1"
    bl_label = 'Export WC1'
    bl_options = {'PRESET'}

    filename_ext = ".wc1"
    filter_glob: StringProperty(
            default="*.wc1",
            options={'HIDDEN'},
            )

    # context group
    use_selection: BoolProperty(
            name="Selection Only",
            description="Export selected objects only",
            default=False,
            )

    # object group
    use_mesh_modifiers: BoolProperty(
            name="Apply Modifiers",
            description="Apply modifiers",
            default=True,
            )
    # Non working in Blender 2.8 currently.
    # ~ use_mesh_modifiers_render: BoolProperty(
            # ~ name="Use Modifiers Render Settings",
            # ~ description="Use render settings when applying modifiers to mesh objects",
            # ~ default=False,
            # ~ )

    # extra data group

    # grouping group

    global_scale: FloatProperty(
            name="Scale",
            min=0.01, max=1000.0,
            default=1.0,
            )

    path_mode: path_reference_mode

    check_extension = True

    def execute(self, context):
        from mathutils import Matrix
        keywords = self.as_keywords(ignore=("axis_forward",
                                            "axis_up",
                                            "global_scale",
                                            "check_existing",
                                            "filter_glob",
                                            ))

        global_matrix = (Matrix.Scale(self.global_scale, 4) @
                         axis_conversion(to_forward=self.axis_forward,
                                         to_up=self.axis_up,
                                         ).to_4x4())

        keywords["global_matrix"] = global_matrix
        return save(context, **keywords)


def menu_func_export(self, context):
    self.layout.operator(ExportWC1.bl_idname, text="WorldClient (.wc1)")


classes = (
    ExportWC1,
)


def register():
    for cls in classes:
        bpy.utils.register_class(cls)

    bpy.types.TOPBAR_MT_file_export.append(menu_func_export)


def unregister():
    bpy.types.TOPBAR_MT_file_export.remove(menu_func_export)

    for cls in classes:
        bpy.utils.unregister_class(cls)


def name_compat(name):
    if name is None:
        return 'None'
    else:
        return name.replace(' ', '_')


def mesh_triangulate(me):
    import bmesh
    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.triangulate(bm, faces=bm.faces)
    bm.to_mesh(me)
    bm.free()

def get_binary_f64(val):
    import struct
    return struct.pack("d", val)

def get_binary_u64(val):
    import struct
    return struct.pack("Q", val)


def write_file(filepath, objects, depsgraph, scene,
               EXPORT_TRI=False,
               EXPORT_EDGES=False,
               EXPORT_SMOOTH_GROUPS=False,
               EXPORT_SMOOTH_GROUPS_BITFLAGS=False,
               EXPORT_NORMALS=False,
               EXPORT_UV=True,
               EXPORT_MTL=True,
               EXPORT_APPLY_MODIFIERS=True,
               EXPORT_APPLY_MODIFIERS_RENDER=False,
               EXPORT_BLEN_OBS=True,
               EXPORT_GROUP_BY_OB=False,
               EXPORT_GROUP_BY_MAT=False,
               EXPORT_KEEP_VERT_ORDER=False,
               EXPORT_POLYGROUPS=False,
               EXPORT_CURVE_AS_NURBS=True,
               EXPORT_GLOBAL_MATRIX=None,
               EXPORT_PATH_MODE='AUTO',
               progress=ProgressReport(),
               EXPORT_VERTEX_COLORS=True,
               ):
    """
    Basic write function. The context and options must be already set
    This can be accessed externaly
    eg.
    write( 'c:\\test\\foobar.obj', Blender.Object.GetSelected() ) # Using default options.
    """
    if EXPORT_GLOBAL_MATRIX is None:
        EXPORT_GLOBAL_MATRIX = Matrix()

    with ProgressReportSubstep(progress, 2, "WC1 Export path: %r" % filepath, "WC1 Export Finished") as subprogress1:
        with open(filepath, "wb") as fhnd:
            fw = fhnd.write

            # Write placeholder Header
            fw(get_binary_u64(0))

            # Initialize totals, these are updated each object
            totverts = totmeshes = 0

            face_vert_index = 1

            copy_set = set()

            # Get all meshes
            subprogress1.enter_substeps(len(objects))
            for i, ob_main in enumerate(objects):
                
                # ignore dupli children
                if ob_main.parent and ob_main.parent.instance_type in {'VERTS', 'FACES'}:
                    subprogress1.step("Ignoring %s, dupli child..." % ob_main.name)
                    continue

                obs = [(ob_main, ob_main.matrix_world)]
                if ob_main.is_instancer:
                    obs += [(dup.instance_object.original, dup.matrix_world.copy())
                            for dup in depsgraph.object_instances
                            if dup.parent and dup.parent.original == ob_main]
                    # ~ print(ob_main.name, 'has', len(obs) - 1, 'dupli children')

                subprogress1.enter_substeps(len(obs))
                
                # write Header
                end_pos = fhnd.tell()
                fhnd.seek(0)
                fw(get_binary_u64(len(obs)))
                fhnd.seek(end_pos)
                    
                for ob, ob_mat in obs:
                    
                    with ProgressReportSubstep(subprogress1, 5) as subprogress2:
                        object_pos = fhnd.tell()
                        
                        # Write placeholder Object Header
                        ## vertex count
                        fw(get_binary_u64(0))
                        ## index count
                        fw(get_binary_u64(0))

                        ## flags
                        flags = 0
                        if EXPORT_VERTEX_COLORS:
                            flags |= 1 << 0
                        fw(get_binary_u64(flags))

                        ob_for_convert = ob.evaluated_get(depsgraph) if EXPORT_APPLY_MODIFIERS else ob.original

                        try:
                            me = ob_for_convert.to_mesh()
                        except RuntimeError:
                            me = None

                        if me is None:
                            continue

                        # _must_ do this before applying transformation, else tessellation may differ
                        if EXPORT_TRI:
                            # _must_ do this first since it re-allocs arrays
                            mesh_triangulate(me)

                        me.transform(EXPORT_GLOBAL_MATRIX @ ob_mat)
                        # If negative scaling, we have to invert the normals...
                        if ob_mat.determinant() < 0.0:
                            me.flip_normals()

                        me_verts = me.vertices[:]
                        me_cols = me.vertex_colors[:]

                        # Make our own list so it can be sorted to reduce context switching
                        face_index_pairs = [(face, index) for index, face in enumerate(me.polygons)]

                        if not (len(face_index_pairs) + len(me.vertices)):  # Make sure there is something to write
                            # clean up
                            bpy.data.meshes.remove(me)
                            continue  # dont bother with this mesh.

                        subprogress2.step()

                        # Vert
                        for v in me_verts:
                            fw(get_binary_f64(v.co[0]))
                            fw(get_binary_f64(-v.co[1]))
                            fw(get_binary_f64(v.co[2]))

                        subprogress2.step()
                        
                        for col_layer in me_cols:
                            for col in col_layer.data:
                                fw(get_binary_f64(col.color[0]))
                                fw(get_binary_f64(col.color[1]))
                                fw(get_binary_f64(col.color[2]))
                                fw(get_binary_f64(col.color[3]))

                        subprogress2.step()

                        obj_indices = 0
                        for f, f_index in face_index_pairs:

                            f_v = [(vi, me_verts[v_idx], l_idx)
                                   for vi, (v_idx, l_idx) in enumerate(zip(f.vertices, f.loop_indices))]

                            for vi, v, li in f_v:
                                fw(get_binary_u64(totverts + v.index))
                                obj_indices += 1

                        subprogress2.step()

                        # Make the indices global rather then per mesh
                        totverts += len(me_verts)
                        
                        # write object header
                        end_pos = fhnd.tell()
                        fhnd.seek(object_pos)
                        fw(get_binary_u64(len(me_verts)))
                        fw(get_binary_u64(obj_indices))
                        fhnd.seek(end_pos)

                        # clean up
                        ob_for_convert.to_mesh_clear()
                    
                subprogress1.leave_substeps("Finished writing geometry of '%s'." % ob_main.name)
            
            subprogress1.leave_substeps()

        subprogress1.step("Finished exporting geometry, now exporting materials")

        # copy all collected files.
        io_utils.path_reference_copy(copy_set)


def _write(context, filepath,
           EXPORT_TRI,  # ok
           EXPORT_EDGES,
           EXPORT_SMOOTH_GROUPS,
           EXPORT_SMOOTH_GROUPS_BITFLAGS,
           EXPORT_NORMALS,  # ok
           EXPORT_UV,  # ok
           EXPORT_MTL,
           EXPORT_APPLY_MODIFIERS,  # ok
           EXPORT_APPLY_MODIFIERS_RENDER,  # ok
           EXPORT_BLEN_OBS,
           EXPORT_GROUP_BY_OB,
           EXPORT_GROUP_BY_MAT,
           EXPORT_KEEP_VERT_ORDER,
           EXPORT_POLYGROUPS,
           EXPORT_CURVE_AS_NURBS,
           EXPORT_SEL_ONLY,  # ok
           EXPORT_ANIMATION,
           EXPORT_GLOBAL_MATRIX,
           EXPORT_PATH_MODE,  # Not used
           ):

    with ProgressReport(context.window_manager) as progress:
        base_name, ext = os.path.splitext(filepath)
        context_name = [base_name, '', '', ext]  # Base name, scene name, frame number, extension

        depsgraph = context.evaluated_depsgraph_get()
        scene = context.scene

        # Exit edit mode before exporting, so current object states are exported properly.
        if bpy.ops.object.mode_set.poll():
            bpy.ops.object.mode_set(mode='OBJECT')

        orig_frame = scene.frame_current

        # Export an animation?
        if EXPORT_ANIMATION:
            scene_frames = range(scene.frame_start, scene.frame_end + 1)  # Up to and including the end frame.
        else:
            scene_frames = [orig_frame]  # Dont export an animation.

        # Loop through all frames in the scene and export.
        progress.enter_substeps(len(scene_frames))
        for frame in scene_frames:
            if EXPORT_ANIMATION:  # Add frame to the filepath.
                context_name[2] = '_%.6d' % frame

            scene.frame_set(frame, subframe=0.0)
            if EXPORT_SEL_ONLY:
                objects = context.selected_objects
            else:
                objects = scene.objects

            full_path = ''.join(context_name)

            # erm... bit of a problem here, this can overwrite files when exporting frames. not too bad.
            # EXPORT THE FILE.
            progress.enter_substeps(1)
            write_file(full_path, objects, depsgraph, scene,
                       EXPORT_TRI,
                       EXPORT_EDGES,
                       EXPORT_SMOOTH_GROUPS,
                       EXPORT_SMOOTH_GROUPS_BITFLAGS,
                       EXPORT_NORMALS,
                       EXPORT_UV,
                       EXPORT_MTL,
                       EXPORT_APPLY_MODIFIERS,
                       EXPORT_APPLY_MODIFIERS_RENDER,
                       EXPORT_BLEN_OBS,
                       EXPORT_GROUP_BY_OB,
                       EXPORT_GROUP_BY_MAT,
                       EXPORT_KEEP_VERT_ORDER,
                       EXPORT_POLYGROUPS,
                       EXPORT_CURVE_AS_NURBS,
                       EXPORT_GLOBAL_MATRIX,
                       EXPORT_PATH_MODE,
                       progress,
                       )
            progress.leave_substeps()

        scene.frame_set(orig_frame, subframe=0.0)
        progress.leave_substeps()


def save(context,
         filepath,
         *,
         use_triangles=False,
         use_edges=True,
         use_normals=False,
         use_smooth_groups=False,
         use_smooth_groups_bitflags=False,
         use_uvs=True,
         use_materials=True,
         use_mesh_modifiers=True,
         use_mesh_modifiers_render=False,
         use_blen_objects=True,
         group_by_object=False,
         group_by_material=False,
         keep_vertex_order=False,
         use_vertex_groups=False,
         use_nurbs=True,
         use_selection=True,
         use_animation=False,
         global_matrix=None,
         path_mode='AUTO'
         ):

    _write(context, filepath,
           EXPORT_TRI=use_triangles,
           EXPORT_EDGES=use_edges,
           EXPORT_SMOOTH_GROUPS=use_smooth_groups,
           EXPORT_SMOOTH_GROUPS_BITFLAGS=use_smooth_groups_bitflags,
           EXPORT_NORMALS=use_normals,
           EXPORT_UV=use_uvs,
           EXPORT_MTL=use_materials,
           EXPORT_APPLY_MODIFIERS=use_mesh_modifiers,
           EXPORT_APPLY_MODIFIERS_RENDER=use_mesh_modifiers_render,
           EXPORT_BLEN_OBS=use_blen_objects,
           EXPORT_GROUP_BY_OB=group_by_object,
           EXPORT_GROUP_BY_MAT=group_by_material,
           EXPORT_KEEP_VERT_ORDER=keep_vertex_order,
           EXPORT_POLYGROUPS=use_vertex_groups,
           EXPORT_CURVE_AS_NURBS=use_nurbs,
           EXPORT_SEL_ONLY=use_selection,
           EXPORT_ANIMATION=use_animation,
           EXPORT_GLOBAL_MATRIX=global_matrix,
           EXPORT_PATH_MODE=path_mode,
           )

    return {'FINISHED'}

if __name__ == "__main__":
    register()
