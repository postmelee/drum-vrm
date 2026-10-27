import { expect, it } from 'vitest';
import { Object3D, Vector3 } from 'three';
import { fitThumbToShaft } from '../src/shaft-grip';
it('balances pad opposition and distal alignment without changing joint lengths',()=>{
  const root=new Object3D(),joints=[new Object3D(),new Object3D(),new Object3D()],pad=new Object3D();
  root.add(joints[0]);joints[0].add(joints[1]);joints[1].add(joints[2]);joints[2].add(pad);
  joints[1].position.set(.03,0,0);joints[2].position.set(.025,0,0);pad.position.set(.018,0,0);
  const positions=[...joints,pad].map(node=>node.position.clone()),target=new Vector3(.05,.035,0),shaft=new Vector3(.8,.6,0);
  root.updateMatrixWorld(true);const before=pad.getWorldPosition(new Vector3()).distanceTo(target);
  fitThumbToShaft(joints,pad,target,shaft);
  expect(pad.getWorldPosition(new Vector3()).distanceTo(target)).toBeLessThan(before*.4);
  const axis=pad.getWorldPosition(new Vector3()).sub(joints[2].getWorldPosition(new Vector3())).normalize();
  expect(axis.dot(shaft)).toBeGreaterThan(.8);
  [...joints,pad].forEach((node,i)=>expect(node.position.toArray()).toEqual(positions[i].toArray()));
});
