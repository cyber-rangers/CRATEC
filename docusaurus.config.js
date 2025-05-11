// @ts-check
import { themes as prismThemes } from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'CRATEC',
  tagline: 'Nástroj pro forenzní vytváření bitových kopií', // Aktualizovaný tagline
  favicon: 'img/favicon.ico',

  // GitHub Pages deployment URL
  url: 'https://cyber-rangers.github.io',
  baseUrl: '/CRATEC/',

  // GitHub repo info
  organizationName: 'cyber-rangers',
  projectName: 'CRATEC',
  deploymentBranch: 'gh-pages',
  trailingSlash: false,

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: './sidebars.js',
          editUrl: 'https://github.com/cyber-rangers/CRATEC/edit/docs/',
        },
        blog: false, // Zakázání blogu
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      image: 'img/docusaurus-social-card.jpg',
      navbar: {
        title: 'CRATEC',
        logo: {
          alt: 'CRATEC Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'tutorialSidebar', // Použijeme hlavní sidebar
            position: 'left',
            label: 'Dokumentace',
          },
          {
            href: 'https://github.com/cyber-rangers/CRATEC',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Getting Started',
                to: '/docs/intro',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Stack Overflow',
                href: 'https://stackoverflow.com/questions/tagged/docusaurus',
              },
              {
                label: 'Discord',
                href: 'https://discordapp.com/invite/docusaurus',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'GitHub',
                href: 'https://github.com/cyber-rangers/CRATEC',
              },
            ],
          },
        ],
        copyright: `Copyright © 2025. All rights reserved.`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
      },
    }),
};

export default config;


