import { persisted } from 'svelte-persisted-store';

export const settings = persisted('settings', {
  language: '',
  reportLanguage: '',
  imageFormat: '',
  hashFormat: '',
  passcode: '',
});
