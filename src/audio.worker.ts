import { synthesizePcm } from './audio';
self.onmessage = ({data}) => {
  try {
    const pcm = synthesizePcm(data);
    self.postMessage({ok: true, pcm}, {transfer: pcm.channels.map(channel => channel.buffer)});
  } catch (error) { self.postMessage({ok: false, error: String(error)}); }
};
