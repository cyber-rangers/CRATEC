// @ts-check

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.

 @type {import('@docusaurus/plugin-content-docs').SidebarsConfig}
 */
const sidebars = {
  tutorialSidebar: [
    {
      type: 'category',
      label: 'Hardware',
      items: [
        'hardware/prehled',
        'hardware/zakladni-deska',
        'hardware/chlazeni',
        'hardware/box',
        'hardware/propojovaci-kabely',
        'hardware/dotykovy-displej',
        'hardware/napajeci-zdroj',
      ],
    },
    {
      type: 'category',
      label: 'Instalace',
      items: [
        'instalace/os',
        'instalace/luks-secureboot',
        'instalace/aplikace',
      ],
    },
    {
      type: 'category',
      label: 'O zařízení',
      items: [
        'o-zarizeni/prehled',
        'o-zarizeni/gui',
        'o-zarizeni/vykonnove-testy',
        'o-zarizeni/nastroje',
      ],
    },
  ],
};

export default sidebars;
