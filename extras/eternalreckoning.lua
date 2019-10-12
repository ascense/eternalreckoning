--  This program is free software; you can redistribute it and/or
--  modify it under the terms of the GNU General Public License
--  as published by the Free Software Foundation; either version 2
--  of the License, or (at your option) any later version.
--
--  This program is distributed in the hope that it will be useful,
--  but WITHOUT ANY WARRANTY; without even the implied warranty of
--  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
--  GNU General Public License for more details.
--
--  You should have received a copy of the GNU General Public License
--  along with this program; if not, write to the Free Software Foundation,
--  Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.

eternalreckoning_protocol = Proto("EternalReckoning", "Eternal Reckoning Protocol")

opcode = ProtoField.uint8("eternalreckoning.opcode", "opcode", base.HEX)

uuid = ProtoField.guid("eternalreckoning.uuid", "uuid", base.HEX)
update_count = ProtoField.uint32("eternalreckoning.update_count", "updates", base.DEC)
component_count = ProtoField.uint32("eternalreckoning.component_count", "components", base.DEC)
component_code = ProtoField.uint8("eternalreckoning.component_code", "component", base.HEX)

health = ProtoField.uint64("eternalreckoning.health", "health", base.DEC)
position = ProtoField.double("eternalreckoning.position", "position", base.DEC)

eternalreckoning_protocol.fields = {
  opcode, -- header
  update_count, uuid, component_count, component_code, -- SV_UPDATE_WORLD
  health, -- HEALTH_COMP
  position -- POSITION_COMP
}

function eternalreckoning_protocol.dissector(buffer, pinfo, tree)
  length = buffer:len()
  if length == 0 then return end
  
  pinfo.cols.protocol = eternalreckoning_protocol.name
  
  local subtree = tree:add(eternalreckoning_protocol, buffer(), "Eternal Reckoning Protocol Data")
  
  local opcode_number = buffer(0,1):le_int()
  local opcode_name = get_opcode_name(opcode_number)
  subtree:add_le(opcode, buffer(0,1)):append_text(" (" .. opcode_name .. ")")
  
  if opcode_name == "SV_CONNECT_RESPONSE" then
    subtree:add_le(uuid, buffer(1,16))
  elseif opcode_name == "SV_UPDATE_WORLD" then
    local count = buffer(1,4):le_uint()
    subtree:add_le(update_count, buffer(1,4))
    
    local offset = 5
    for i = 0, count - 1, 1 do
      subtree:add_le(uuid, buffer(offset,16))
      offset = offset + 16

      local components = buffer(offset,4):le_uint()
      subtree:add_le(component_count, buffer(offset,4))
      offset = offset + 4

      for j = 0, components - 1, 1 do
        local component_number = buffer(offset,1):le_int()
        local component_name = get_component_name(component_number)
        subtree:add_le(component_code, buffer(offset,1)):append_text(" (" .. component_name .. ")")
        offset = offset + 1
        
        if component_name == "HEALTH_COMP" then
          subtree:add_le(health, buffer(offset,8))
          offset = offset + 8
        elseif component_name == "POSITION_COMP" then
          subtree:add_le(position, buffer(offset,8)):append_text(" (x)")
          subtree:add_le(position, buffer(offset+8,8)):append_text(" (y)")
          subtree:add_le(position, buffer(offset+16,8)):append_text(" (z)")
          offset = offset + 24
        end
      end
    end
  elseif opcode_name == "CL_MOVE_SET_POSITION" then
    subtree:add_le(position, buffer(1,8)):append_text(" (x)")
    subtree:add_le(position, buffer(9,8)):append_text(" (y)")
    subtree:add_le(position, buffer(17,8)):append_text(" (z)")
  end
end

function get_opcode_name(opcode)
  local opcode_name = "Unknown"
  
      if opcode == 0x01 then opcode_name = "CL_CONNECT_MESSAGE"
  elseif opcode == 0x02 then opcode_name = "SV_CONNECT_RESPONSE"
  elseif opcode == 0x10 then opcode_name = "SV_UPDATE_WORLD"
  elseif opcode == 0x20 then opcode_name = "CL_MOVE_SET_POSITION"
  end
  
  return opcode_name
end

function get_component_name(code)
  local component_name = "Unknown"
  
      if code == 0x01 then component_name = "HEALTH_COMP"
  elseif code == 0x02 then component_name = "POSITION_COMP"
  end
  
  return component_name
end

local tcp_port = DissectorTable.get("tcp.port")
tcp_port:add(6142, eternalreckoning_protocol)