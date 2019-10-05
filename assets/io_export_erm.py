import os

import bpy
from mathutils import Matrix

from bpy_extras.wm_utils.progress_report import (
    ProgressReport,
    ProgressReportSubstep,
)

bl_info = {
    "name": "Eternal Reckoning Model",
    "author": "Henry Carlson",
    "version": (0, 1, 0),
    "blender": (2, 80, 0),
    "location": "File > Import-Export",
    "description": "Import-Export Eternal Reckoning Model",
    "warning": "",
    "wiki_url": "",
    "category": "Import-Export",
}

if "bpy" in locals():
    import importlib
    if "export_wc1" in locals():
        importlib.reload(export_obj)

from bpy.props import (
    BoolProperty,
    FloatProperty,
    StringProperty,
)
from bpy_extras.io_utils import (
    ExportHelper,
    orientation_helper,
    axis_conversion,
)

@orientation_helper(axis_forward='-Z', axis_up='-Y')
class ExportERM(bpy.types.Operator, ExportHelper):
    """Export Eternal Reckoning Model"""

    bl_idname = "model.erm"
    bl_label = "Export ERM"
    bl_options = {'PRESET'}

    filename_ext = ".erm"
    filter_glob: StringProperty(
        default="*.erm",
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

    use_vertex_colors: BoolProperty(
        name="Vertex Colors",
        description="Export vertex colors",
        default=True,
    )

    global_scale: FloatProperty(
        name="Scale",
        min=0.01, max=1000.0,
        default=1.0,
    )

    check_extension = True

    def execute(self, context):
        keywords = self.as_keywords(
            ignore=(
                "axis_forward",
                "axis_up",
                "check_existing",
                "global_scale",
                "filter_glob",
            )
        )

        global_matrix = (
            Matrix.Scale(self.global_scale, 4) @
                axis_conversion(
                    to_forward=self.axis_forward,
                    to_up=self.axis_up,
                ).to_4x4()
        )

        keywords["global_matrix"] = global_matrix
        return save(context, **keywords)

def menu_func_export(self, context):
    self.layout.operator(ExportERM.bl_idname, text="Eternal Reckoning (.erm)")

def register():
    bpy.utils.register_class(ExportERM)
    bpy.types.TOPBAR_MT_file_export.append(menu_func_export)

def unregister():
    bpy.types.TOPBAR_MT_file_export.remove(menu_func_export)
    bpy.utils.unregister_class(ExportERM)

def name_compat(name):
    if name is None:
        return 'None'
    return name

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

def write_file(
    filepath, objects, depsgraph, scene,
    use_mesh_modifiers=True,
    use_vertex_colors=True,
    global_matrix=None,
    progress=ProgressReport(),
):
    if global_matrix is None:
        global_matrix = Matrix()

    with ProgressReportSubstep(progress, 1, "ERM Export path: %r" % filepath, "ERM Export Finished") as obj_progress:
        with open(filepath, "wb") as fhnd:
            fw = fhnd.write

            # Initialize totals
            totverts = totmeshes = 0

            objs = [obj for obj in objects if obj.type == 'MESH']

            # write header
            fw(get_binary_u64(len(objs)))

            # Get all meshes
            obj_progress.enter_substeps(len(objs))
            for obj in objs:
                with ProgressReportSubstep(obj_progress, 5) as mesh_progress:
                    object_pos = fhnd.tell()

                    # Write placeholder Object Header
                    ## vertex count
                    fw(get_binary_u64(0))
                    ## index count
                    fw(get_binary_u64(0))
                    ## flags
                    flags = 0
                    if use_vertex_colors:
                        flags |= 1 << 0
                    fw(get_binary_u64(flags))

                    try:
                        mesh = (obj.evaluated_get(depsgraph) if use_mesh_modifiers else obj.original).to_mesh()
                    except RuntimeError:
                        continue

                    mesh_triangulate(mesh)

                    obj_matrix = obj.matrix_world
                    mesh.transform(global_matrix @ obj_matrix)
                    if obj_matrix.determinant() < 0.0:
                        mesh.flip_normals()

                    mesh_verts = mesh.vertices[:]
                    mesh_colors = mesh.vertex_colors[:]

                    if len(mesh.polygons) + len(mesh_verts) <= 0:
                        bpy.data.meshes.remove(mesh)
                        continue

                    # cleanup done
                    mesh_progress.step()

                    # Write verts
                    for v in mesh_verts:
                        fw(get_binary_f64(v.co[0]))
                        fw(get_binary_f64(v.co[1]))
                        fw(get_binary_f64(v.co[2]))

                    # vertices done
                    mesh_progress.step()

                    # Write vertex colors
                    for col_layer in mesh_colors:
                        for col in col_layer.data:
                            fw(get_binary_f64(col.color[0]))
                            fw(get_binary_f64(col.color[1]))
                            fw(get_binary_f64(col.color[2]))
                            fw(get_binary_f64(col.color[3]))

                    # vertex colors done
                    mesh_progress.step()

                    # Write vertex indices
                    obj_indices = 0
                    for face in mesh.polygons:
                        for vert in face.vertices:
                            fw(get_binary_u64(totverts + mesh_verts[vert].index))
                            obj_indices += 1

                    # indices done
                    mesh_progress.step()

                    # Make the indices global rather then per mesh
                    totverts += len(mesh_verts)

                    # Write object header
                    end_pos = fhnd.tell()
                    fhnd.seek(object_pos)
                    fw(get_binary_u64(len(mesh_verts)))
                    fw(get_binary_u64(obj_indices))
                    fhnd.seek(end_pos)
        obj_progress.leave_substeps()

def save(
    context,
    filepath,
    *,
    use_mesh_modifiers=True,
    use_selection=True,
    use_vertex_colors=True,
    global_matrix=None,
):
    with ProgressReport(context.window_manager) as progress:
        # exit edit mode
        if bpy.ops.object.mode_set.poll():
            bpy.ops.object.mode_set(mode='OBJECT')

        depsgraph = context.evaluated_depsgraph_get()
        scene = context.scene

        if use_selection:
            objects = context.selected_objects
        else:
            objects = scene.objects

        progress.enter_substeps(1)
        write_file(
            filepath, objects, depsgraph, scene,
            use_mesh_modifiers=use_mesh_modifiers,
            use_vertex_colors=use_vertex_colors,
            global_matrix=global_matrix,
            progress=progress
        )
        progress.leave_substeps()

    return {'FINISHED'}

if __name__ == "__main__":
    register()