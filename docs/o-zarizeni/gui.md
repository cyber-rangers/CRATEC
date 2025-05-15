---
title: GUI
---

import React from 'react';
import { PhotoProvider, PhotoView } from 'react-photo-view';
import 'react-photo-view/dist/react-photo-view.css';

# GUI

Níže jsou zobrazeny ukázky GUI aplikace:

<PhotoProvider>
  <div className="row">
    {Array.from({ length: 21 }, (_, i) => (
      <div className="col col--4 margin-bottom--lg" key={i + 1}>
        <PhotoView src={`/CRATEC/img/GUI/${i + 1}.png`}>
          <img
            src={`/CRATEC/img/GUI/${i + 1}.png`}
            alt={`GUI Screenshot ${i + 1}`}
            style={{
              width: '100%',
              borderRadius: '10px',
              boxShadow: '0 4px 8px rgba(0, 0, 0, 0.2)',
              cursor: 'pointer',
            }}
          />
        </PhotoView>
      </div>
    ))}
  </div>
</PhotoProvider>