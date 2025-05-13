import React from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import { Swiper, SwiperSlide } from 'swiper/react';
import 'swiper/css';
import 'swiper/css/navigation';
import 'swiper/css/pagination';
import { Navigation, Pagination, Autoplay } from 'swiper/modules';
import styles from './index.module.css';

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <h1
          className="hero__title"
          style={{ color: '#000' }} // Nastavení barvy hlavního nápisu na černou
        >
          {siteConfig.title}
        </h1>
        <p
          className="hero__subtitle"
          style={{ color: '#000' }} // Nastavení barvy textu pod nápisem na černou
        >
          {siteConfig.tagline}
        </p>
        <div style={{ marginTop: '2rem' }}>
          <Swiper
            modules={[Navigation, Pagination, Autoplay]}
            navigation
            pagination={{ clickable: true }}
            autoplay={{ delay: 3000 }}
            loop
            style={{
              maxWidth: '600px',
              margin: '0 auto',
              position: 'relative',
            }}
          >
            <SwiperSlide>
              <img
                src="/CRATEC/img/box/box1.png"
                alt="Zařízení - Box 1"
                style={{ width: '100%', borderRadius: '10px' }}
              />
            </SwiperSlide>
            <SwiperSlide>
              <img
                src="/CRATEC/img/box/box2.png"
                alt="Zařízení - Box 2"
                style={{ width: '100%', borderRadius: '10px' }}
              />
            </SwiperSlide>
          </Swiper>
        </div>
      </div>
    </header>
  );
}

export default function Home() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title="Dokumentace CRATEC" // Změna title v liště
      description="Nástroj pro forenzní zobrazování a dokumentaci" // Překlad popisu
    >
      <HomepageHeader />
      <footer
        style={{
          backgroundColor: '#000',
          color: '#fff',
          padding: '2rem 0',
          textAlign: 'center',
        }}
      >
        <Link
          className="button button--secondary button--lg"
          to="/docs/o-zarizeni/prehled"
          style={{
            backgroundColor: '#fff',
            color: '#000',
            textDecoration: 'none',
            padding: '1rem 2rem',
            borderRadius: '5px',
          }}
        >
          Přejít do dokumentace
        </Link>
      </footer>
    </Layout>
  );
}
