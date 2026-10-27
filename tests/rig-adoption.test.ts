import { describe, expect, it, vi } from 'vitest';
import { Group } from 'three';
import { DrumScene } from '../src/scene';
import { NativeHumanoidRigAdapter } from '../src/rig-humanoid';

function session(){
  const previous=new NativeHumanoidRigAdapter(),stage=new Group(),scene=Object.create(DrumScene.prototype) as DrumScene;
  stage.add(previous.root);
  Object.assign(scene,{rig:previous,scene:stage,kit:[],plan:null,stickTrace:new Group(),characterVisible:true,skeletonVisible:true,modelVisible:false,editorMode:false,cameraView:'stage',theme:'light'});
  return {previous,stage,scene};
}
describe('character adoption ownership',()=>{
  it.each(['calibrate','applyPose'] as const)('keeps the previous character alive when candidate %s fails',method=>{
    const {previous,stage,scene}=session(),candidate=new NativeHumanoidRigAdapter();
    const dispose=vi.spyOn(previous,'dispose');
    vi.spyOn(candidate,method).mockImplementation(()=>{throw new Error('candidate preparation failed');});
    expect(()=>scene.setRig(candidate)).toThrow('candidate preparation failed');
    expect(dispose).not.toHaveBeenCalled();
    expect(stage.children).toContain(previous.root);expect(stage.children).not.toContain(candidate.root);
    expect(scene.getRigProfile()).toEqual(previous.getProfile());
    previous.dispose();candidate.dispose();
  });
  it('disposes the old rig only after a replacement is configured',()=>{
    const {previous,stage,scene}=session(),candidate=new NativeHumanoidRigAdapter();
    const dispose=vi.spyOn(previous,'dispose'),configure=vi.spyOn(candidate,'setVisualOptions');
    scene.setRig(candidate);
    expect(configure.mock.invocationCallOrder[0]).toBeLessThan(dispose.mock.invocationCallOrder[0]);
    expect(stage.children).toContain(candidate.root);expect(stage.children).not.toContain(previous.root);
    expect(scene.getRigProfile()).toEqual(candidate.getProfile());candidate.dispose();
  });
  it('measures a candidate calibration without leaving changes in the displayed rig',()=>{
    const {previous,scene}=session(),profile=scene.getRigProfile(),calibration=scene.getCalibration();
    const contacts=previous.getContactSamples();
    const candidate=scene.measureCalibration({...calibration,seatHeight:calibration.seatHeight+.05,palmOrientation:'down'},[]);
    expect(candidate).not.toEqual(profile);
    expect(JSON.stringify(scene.getRigProfile())).toBe(JSON.stringify(profile));expect(scene.getCalibration()).toEqual(calibration);
    expect(previous.getContactSamples()).toEqual(contacts);
    // The displayed rig still owns its yawed default pedal, even while the
    // scene/candidate kit is empty. Future omitted-kit calls retain that frame.
    expect(previous.setCalibration(calibration).bendPoles[2]).toEqual(profile.bendPoles[2]);
    expect(()=>scene.measureCalibration({...calibration,seatHeight:10},[])).toThrow();
    expect(JSON.stringify(scene.getRigProfile())).toBe(JSON.stringify(profile));previous.dispose();
  });
});
