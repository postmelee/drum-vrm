import { Euler, Quaternion, MathUtils } from 'three';
import { humanoidDocument } from './vrm';

// Independently authored skeletal FBX; no downloaded geometry or motion data.
export interface FBXNode { name: string; props?: (string | number | bigint | Uint8Array)[]; children?: FBXNode[]; }
export function encodeFBX(nodes: FBXNode[], version: 7400 | 7500 = 7400): ArrayBuffer {
  const wide = version === 7500, head = wide ? 25 : 13, enc = new TextEncoder();
  const property = (value: string | number | bigint | Uint8Array) => {
    if (value instanceof Uint8Array) return value;
    const string = typeof value === 'string' ? enc.encode(value) : null;
    const bytes = new Uint8Array(string ? 5 + string.length : 9), view = new DataView(bytes.buffer);
    bytes[0] = string ? 83 : typeof value === 'bigint' ? 76 : 68;
    if (string) { view.setUint32(1, string.length, true); bytes.set(string, 5); }
    else if (typeof value === 'bigint') view.setBigInt64(1, value, true); else view.setFloat64(1, value as number, true);
    return bytes;
  };
  const size = (node: FBXNode): number => head + enc.encode(node.name).length + (node.props ?? []).reduce<number>((n, p) => n + property(p).length, 0) + (node.children?.length ? node.children.reduce((n, child) => n + size(child), head) : 0);
  const result = new Uint8Array(27 + nodes.reduce((n, node) => n + size(node), 0) + head + 176), view = new DataView(result.buffer);
  result.set(enc.encode('Kaydara FBX Binary  \0\x1a\0')); view.setUint32(23, version, true);
  const uint = (offset: number, value: number) => wide ? view.setBigUint64(offset, BigInt(value), true) : view.setUint32(offset, value, true);
  const write = (node: FBXNode, offset: number): number => {
    const end = offset + size(node), props = (node.props ?? []).map(property), name = enc.encode(node.name);
    uint(offset, end); uint(offset + (wide ? 8 : 4), props.length); uint(offset + (wide ? 16 : 8), props.reduce((n, p) => n + p.length, 0)); result[offset + head - 1] = name.length;
    let cursor = offset + head; result.set(name, cursor); cursor += name.length;
    for (const prop of props) { result.set(prop, cursor); cursor += prop.length; }
    for (const child of node.children ?? []) cursor = write(child, cursor);
    return end;
  };
  let cursor = 27; for (const node of nodes) cursor = write(node, cursor);
  return result.buffer;
}
export function fbxDocument(units = 100, facing: '0.0' | '1.0' = '1.0', names: Record<string, string> = {}): FBXNode[] {
  const source = humanoidDocument(facing, true, units);
  const prop = (name: string, kind: string, values: number[]): FBXNode => ({ name: 'P', props: [name, kind, '', 'A', ...values] });
  return [
    { name: 'GlobalSettings', children: [{ name: 'Properties70', children: [prop('UpAxis', 'int', [1]), prop('UnitScaleFactor', 'double', [100 / units])] }] },
    { name: 'Objects', children: source.nodes.map((node, i) => {
      const euler = new Euler().setFromQuaternion(new Quaternion(...node.rotation), 'ZYX');
      return { name: 'Model', props: [BigInt(i + 1), `${names[node.name] ?? node.name}\0\x01Model`, 'LimbNode'], children: [{ name: 'Properties70', children: [
        prop('Lcl Translation', 'Lcl Translation', node.translation), prop('Lcl Rotation', 'Lcl Rotation', [euler.x, euler.y, euler.z].map(MathUtils.radToDeg)), prop('Lcl Scaling', 'Lcl Scaling', [1, 1, 1]),
      ] }] };
    }) },
    { name: 'Connections', children: [{ name: 'C', props: ['OO', 1n, 0n] }, ...source.nodes.flatMap((node, i) => node.children.map(child => ({ name: 'C', props: ['OO', BigInt(child + 1), BigInt(i + 1)] }))) ] },
  ];
}
